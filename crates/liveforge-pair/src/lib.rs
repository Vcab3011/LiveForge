use liveforge_metadata::{LivePhotoIdentity, MetadataPlan};
use std::{
    error::Error as StdError,
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub type BackendResult<T> = Result<T, Box<dyn StdError + Send + Sync>>;

#[derive(Debug, Clone, Copy)]
pub struct PairRequest<'a> {
    pub source_video: &'a Path,
    pub output_directory: &'a Path,
    pub output_name: &'a str,
    pub still_time_seconds: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LivePhotoResources {
    pub directory: PathBuf,
    pub photo: PathBuf,
    pub motion: PathBuf,
    pub metadata: MetadataPlan,
}

pub trait PairBackend {
    fn photo_extension(&self) -> &'static str;

    fn write_photo(
        &self,
        source_video: &Path,
        destination: &Path,
        metadata: &MetadataPlan,
    ) -> BackendResult<()>;

    fn write_motion(
        &self,
        source_video: &Path,
        destination: &Path,
        metadata: &MetadataPlan,
    ) -> BackendResult<()>;
}

pub trait PairVerifier {
    fn verify(&self, resources: &LivePhotoResources) -> BackendResult<()>;
}

#[derive(Debug, Error)]
pub enum PairBuildError {
    #[error("source video does not exist: {0}")]
    MissingSource(PathBuf),
    #[error("still-image time must be finite and greater than or equal to zero")]
    InvalidStillTime,
    #[error("output name must contain only ASCII letters, digits, '-' or '_'")]
    InvalidOutputName,
    #[error("output already exists: {0}")]
    OutputExists(PathBuf),
    #[error("filesystem operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("resource writer failed: {0}")]
    Writer(#[source] Box<dyn StdError + Send + Sync>),
    #[error("generated resource verification failed: {0}")]
    Verification(#[source] Box<dyn StdError + Send + Sync>),
}

pub struct PairBuilder<B, V> {
    backend: B,
    verifier: V,
}

impl<B, V> PairBuilder<B, V>
where
    B: PairBackend,
    V: PairVerifier,
{
    pub fn new(backend: B, verifier: V) -> Self {
        Self { backend, verifier }
    }

    pub fn build(&self, request: PairRequest<'_>) -> Result<LivePhotoResources, PairBuildError> {
        validate_request(&request)?;
        fs::create_dir_all(request.output_directory)?;

        let final_directory = request.output_directory.join(request.output_name);
        if final_directory.exists() {
            return Err(PairBuildError::OutputExists(final_directory));
        }

        let identity = LivePhotoIdentity::generate();
        let staging_directory = request.output_directory.join(format!(
            ".{}.liveforge-{}",
            request.output_name, identity.asset_identifier
        ));
        fs::create_dir(&staging_directory)?;

        let result = self.build_in_staging(&request, identity, &staging_directory);
        match result {
            Ok(staged) => {
                if let Err(error) = fs::rename(&staging_directory, &final_directory) {
                    let _ = fs::remove_dir_all(&staging_directory);
                    return Err(PairBuildError::Io(error));
                }
                Ok(LivePhotoResources {
                    directory: final_directory.clone(),
                    photo: final_directory.join(file_name(&staged.photo)),
                    motion: final_directory.join(file_name(&staged.motion)),
                    metadata: staged.metadata,
                })
            }
            Err(error) => {
                let _ = fs::remove_dir_all(&staging_directory);
                Err(error)
            }
        }
    }

    fn build_in_staging(
        &self,
        request: &PairRequest<'_>,
        identity: LivePhotoIdentity,
        staging_directory: &Path,
    ) -> Result<LivePhotoResources, PairBuildError> {
        let metadata = MetadataPlan {
            identity,
            still_image_time_seconds: request.still_time_seconds,
        };
        let stem = format!("IMG_{}", metadata.identity.asset_identifier);
        let photo = staging_directory.join(format!("{}.{}", stem, self.backend.photo_extension()));
        let motion = staging_directory.join(format!("{}.mov", stem));

        self.backend
            .write_photo(request.source_video, &photo, &metadata)
            .map_err(PairBuildError::Writer)?;
        ensure_nonempty(&photo)?;

        self.backend
            .write_motion(request.source_video, &motion, &metadata)
            .map_err(PairBuildError::Writer)?;
        ensure_nonempty(&motion)?;

        let resources = LivePhotoResources {
            directory: staging_directory.to_path_buf(),
            photo,
            motion,
            metadata,
        };
        self.verifier
            .verify(&resources)
            .map_err(PairBuildError::Verification)?;
        Ok(resources)
    }
}

fn validate_request(request: &PairRequest<'_>) -> Result<(), PairBuildError> {
    if !request.source_video.is_file() {
        return Err(PairBuildError::MissingSource(
            request.source_video.to_path_buf(),
        ));
    }
    if !request.still_time_seconds.is_finite() || request.still_time_seconds < 0.0 {
        return Err(PairBuildError::InvalidStillTime);
    }
    if request.output_name.is_empty()
        || !request
            .output_name
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_'))
    {
        return Err(PairBuildError::InvalidOutputName);
    }
    Ok(())
}

fn ensure_nonempty(path: &Path) -> Result<(), PairBuildError> {
    let metadata = fs::metadata(path)?;
    if metadata.len() == 0 {
        return Err(PairBuildError::Writer(
            io::Error::other(format!(
                "writer created an empty resource: {}",
                path.display()
            ))
            .into(),
        ));
    }
    Ok(())
}

fn file_name(path: &Path) -> &std::ffi::OsStr {
    path.file_name()
        .expect("builder-created resource paths always have a file name")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct RecordingBackend {
        fail_motion: bool,
    }

    impl PairBackend for RecordingBackend {
        fn photo_extension(&self) -> &'static str {
            "jpg"
        }

        fn write_photo(
            &self,
            _source_video: &Path,
            destination: &Path,
            metadata: &MetadataPlan,
        ) -> BackendResult<()> {
            fs::write(destination, &metadata.identity.asset_identifier)?;
            Ok(())
        }

        fn write_motion(
            &self,
            _source_video: &Path,
            destination: &Path,
            metadata: &MetadataPlan,
        ) -> BackendResult<()> {
            if self.fail_motion {
                return Err(io::Error::other("injected motion failure").into());
            }
            fs::write(destination, &metadata.identity.asset_identifier)?;
            Ok(())
        }
    }

    struct IdentifierVerifier;

    impl PairVerifier for IdentifierVerifier {
        fn verify(&self, resources: &LivePhotoResources) -> BackendResult<()> {
            let photo_identifier = fs::read_to_string(&resources.photo)?;
            let motion_identifier = fs::read_to_string(&resources.motion)?;
            if photo_identifier != motion_identifier
                || photo_identifier != resources.metadata.identity.asset_identifier
            {
                return Err(io::Error::other("identifiers do not match").into());
            }
            Ok(())
        }
    }

    fn request<'a>(source: &'a Path, output: &'a Path) -> PairRequest<'a> {
        PairRequest {
            source_video: source,
            output_directory: output,
            output_name: "IMG_0001",
            still_time_seconds: 1.5,
        }
    }

    #[test]
    fn publishes_a_verified_pair_as_one_directory() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let source = temporary.path().join("source.mov");
        fs::write(&source, b"video").expect("source fixture");
        let output = temporary.path().join("output");
        let builder = PairBuilder::new(RecordingBackend { fail_motion: false }, IdentifierVerifier);

        let resources = builder
            .build(request(&source, &output))
            .expect("pair should be published");

        assert_eq!(resources.directory, output.join("IMG_0001"));
        assert!(resources.photo.is_file());
        assert!(resources.motion.is_file());
        assert_eq!(
            fs::read_to_string(resources.photo).expect("photo identifier"),
            fs::read_to_string(resources.motion).expect("motion identifier")
        );
    }

    #[test]
    fn removes_staging_output_when_a_writer_fails() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let source = temporary.path().join("source.mov");
        fs::write(&source, b"video").expect("source fixture");
        let output = temporary.path().join("output");
        let builder = PairBuilder::new(RecordingBackend { fail_motion: true }, IdentifierVerifier);

        assert!(builder.build(request(&source, &output)).is_err());
        assert!(!output.join("IMG_0001").exists());
        assert_eq!(fs::read_dir(output).expect("output directory").count(), 0);
    }

    #[test]
    fn refuses_to_overwrite_an_existing_pair() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let source = temporary.path().join("source.mov");
        fs::write(&source, b"video").expect("source fixture");
        let output = temporary.path().join("output");
        fs::create_dir_all(output.join("IMG_0001")).expect("existing output");
        let builder = PairBuilder::new(RecordingBackend { fail_motion: false }, IdentifierVerifier);

        let error = builder
            .build(request(&source, &output))
            .expect_err("overwrite must be rejected");
        assert!(matches!(error, PairBuildError::OutputExists(_)));
    }
}
