use crate::config::LimageConfig;
use std::{
    path::Path,
    process::{Command, Stdio},
};
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

pub struct Builder {
    config: LimageConfig,
}

impl Builder {
    pub fn new(config: LimageConfig) -> Result<Self, BuildError> {
        debug!("Creating new Builder with config: {:?}", config);
        Ok(Self { config })
    }

    #[instrument(skip(self), err)]
    pub fn build(&self, kernel_path: Option<&Path>) -> Result<(), BuildError> {
        info!("Starting build process");
        self.execute_prebuilder()?;
        self.prepare_ovmf_files()?;
        self.prepare_limine_files()?;
        self.copy_kernel(kernel_path)?;
        self.create_limine_iso()?;
        info!("Build completed successfully");
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn execute_prebuilder(&self) -> Result<(), BuildError> {
        if let Some(cmd) = &self.config.build.prebuilder {
            info!("Executing prebuilder command: {}", cmd);
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .stdout(Stdio::piped())
                .output()
                .map_err(|e| BuildError::PrebuilderFailed { source: e })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Prebuilder command exited with non-zero status: {}", stderr);
            } else {
                debug!("Prebuilder executed successfully");
            }
        } else {
            debug!("No prebuilder command specified, skipping");
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn prepare_ovmf_files(&self) -> Result<(), BuildError> {
        info!("Preparing OVMF files in: {:?}", self.config.build.ovmf_path);
        std::fs::create_dir_all(&self.config.build.ovmf_path)?;

        for arch in &["x86_64"] {
            for kind in &["code", "vars"] {
                let url = format!(
                    "https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/ovmf-{}-{}.fd",
                    kind, arch
                );
                let path = self
                    .config
                    .build
                    .ovmf_path
                    .join(format!("ovmf-{}-{}.fd", kind, arch));

                debug!("Downloading OVMF file from {} to {:?}", url, path);
                let result = Command::new("curl")
                    .arg("-Lo")
                    .arg(&path)
                    .arg(&url)
                    .stdout(Stdio::piped())
                    .output()
                    .map_err(|e| BuildError::DownloadOvmfFailed { source: e });

                if let Err(e) = &result {
                    error!("Failed to download OVMF file: {}", e);
                }
                result?;
                info!("Downloaded OVMF {}-{}.fd successfully", kind, arch);
            }
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn prepare_limine_files(&self) -> Result<(), BuildError> {
        info!("Preparing Limine files");
        self.clone_limine_binary()?;
        self.copy_limine_config()?;
        self.copy_limine_binary()?;
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn clone_limine_binary(&self) -> Result<(), BuildError> {
        let required_files = [
            "limine-bios.sys",
            "limine-bios-cd.bin",
            "limine-uefi-cd.bin",
            "BOOTX64.EFI",
            "BOOTIA32.EFI",
        ];

        let should_clone = !self.config.build.limine_path.exists()
            || required_files.iter().any(|file| {
                let file_path = self.config.build.limine_path.join(file);
                !file_path.exists()
            });

        if should_clone {
            // If directory exists but is incomplete, remove it first
            if self.config.build.limine_path.exists() {
                info!(
                    "Limine directory exists but missing required files, removing and re-cloning"
                );
                std::fs::remove_dir_all(&self.config.build.limine_path).map_err(|e| {
                    BuildError::CloneLimineFailed {
                        source: std::io::Error::new(
                            e.kind(),
                            format!("Failed to remove incomplete Limine directory: {}", e),
                        ),
                    }
                })?;
            } else {
                info!(
                    "Cloning Limine repository to {:?}",
                    self.config.build.limine_path
                );
            }

            std::fs::create_dir_all(&self.config.build.limine_path)?; // Create first
            let clone_result = Command::new("git")
                .args(&[
                    "clone",
                    "https://github.com/limine-bootloader/limine.git",
                    "--branch=v8.x-binary",
                    "--depth=1",
                ])
                .arg(&self.config.build.limine_path)
                .stdout(Stdio::piped())
                .output()
                .map_err(|e| BuildError::CloneLimineFailed { source: e });

            if let Err(e) = &clone_result {
                error!("Failed to clone Limine repository: {}", e);
            }
            clone_result?;

            info!("Building Limine");
            let build_result = Command::new("make")
                .arg("-C")
                .arg(&self.config.build.limine_path)
                .status()
                .map_err(|e| BuildError::CloneLimineFailed { source: (e) });

            if let Err(e) = &build_result {
                error!("Failed to build Limine: {}", e);
            }
            build_result?;

            info!("Limine repository cloned and built successfully");
        } else {
            debug!("Limine repository exists with all required files, skipping clone");
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn copy_limine_config(&self) -> Result<(), BuildError> {
        let config_dir = self.config.build.iso_root.join("boot").join("limine");
        debug!("Creating Limine config directory: {:?}", config_dir);
        std::fs::create_dir_all(&config_dir)?;

        info!("Copying limine.conf to {:?}", config_dir);
        std::fs::copy("limine.conf", config_dir.join("limine.conf"))
            .map_err(|e| BuildError::CopyLimineConfig { source: e })?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    fn copy_limine_binary(&self) -> Result<(), BuildError> {
        let limine_boot_dir = self.config.build.iso_root.join("boot").join("limine");
        let limine_efi_dir = self.config.build.iso_root.join("EFI").join("BOOT");

        debug!(
            "Creating Limine binary directories: {:?} and {:?}",
            limine_boot_dir, limine_efi_dir
        );
        std::fs::create_dir_all(&limine_boot_dir)?;
        std::fs::create_dir_all(&limine_efi_dir)?;

        // Copy BIOS files
        info!("Copying Limine BIOS files");
        for file in &[
            "limine-bios.sys",
            "limine-bios-cd.bin",
            "limine-uefi-cd.bin",
        ] {
            let src = self.config.build.limine_path.join(file);
            let dst = limine_boot_dir.join(file);
            debug!("Copying {} from {:?} to {:?}", file, src, dst);

            std::fs::copy(&src, &dst).map_err(|e| BuildError::CopyLimineBinary {
                file: file.to_string(),
                source: e,
            })?;
        }

        // Copy UEFI files
        info!("Copying Limine UEFI files");
        for file in &["BOOTX64.EFI", "BOOTIA32.EFI"] {
            let src = self.config.build.limine_path.join(file);
            let dst = limine_efi_dir.join(file);
            debug!("Copying {} from {:?} to {:?}", file, src, dst);

            std::fs::copy(&src, &dst).map_err(|e| BuildError::CopyLimineBinary {
                file: file.to_string(),
                source: e,
            })?;
        }

        Ok(())
    }

    #[instrument(skip(self), err)]
    fn copy_kernel(&self, kernel_path: Option<&Path>) -> Result<(), BuildError> {
        let kernel_dir = self.config.build.iso_root.join("boot").join("kernel");
        debug!("Creating kernel directory: {:?}", kernel_dir);
        std::fs::create_dir_all(&kernel_dir)?;

        let kernel_binary =
            kernel_path.unwrap_or_else(|| Path::new("target/x86_64-unknown-none/debug/kernel"));

        info!(
            "Copying kernel from {:?} to {:?}",
            kernel_binary,
            kernel_dir.join("kernel")
        );
        std::fs::copy(kernel_binary, kernel_dir.join("kernel"))
            .map_err(|e| BuildError::CopyKernel { source: e })?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    fn create_limine_iso(&self) -> Result<(), BuildError> {
        // Create parent directory for the ISO if it doesn't exist
        if let Some(parent) = self.config.build.image_path.parent() {
            debug!("Creating parent directory for ISO: {:?}", parent);
            std::fs::create_dir_all(parent)?;
        }

        self.create_raw_iso()?;
        self.install_limine_to_iso()?;
        info!("ISO creation completed");
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn create_raw_iso(&self) -> Result<(), BuildError> {
        info!("Creating raw ISO at {:?}", self.config.build.image_path);
        let result = Command::new("xorriso")
            .args(&[
                "-as",
                "mkisofs",
                "-b",
                "boot/limine/limine-bios-cd.bin",
                "-no-emul-boot",
                "-boot-load-size",
                "4",
                "-boot-info-table",
                "--efi-boot",
                "boot/limine/limine-uefi-cd.bin",
                "-efi-boot-part",
                "--efi-boot-image",
                "--protective-msdos-label",
            ])
            .arg(&self.config.build.iso_root)
            .arg("-o")
            .arg(&self.config.build.image_path)
            .stdout(Stdio::piped())
            .output()
            .map_err(|e| BuildError::CreateIso { source: e });

        if let Err(e) = &result {
            error!("Failed to create ISO: {}", e);
        }
        result?;
        debug!("Raw ISO created successfully");
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn install_limine_to_iso(&self) -> Result<(), BuildError> {
        let limine_binary = self.config.build.limine_path.join("limine");
        info!("Installing Limine to ISO using binary: {:?}", limine_binary);
        let result = Command::new(limine_binary)
            .args(&[
                "bios-install",
                &self.config.build.image_path.display().to_string(),
            ])
            .stdout(Stdio::piped())
            .output()
            .map_err(|e| BuildError::InstallLimine { source: e });

        if let Err(e) = &result {
            error!("Failed to install Limine to ISO: {}", e);
        }
        result?;
        info!("Limine installed to ISO successfully");
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Failed to locate Cargo.toml")]
    LocateManifest(#[from] locate_cargo_manifest::LocateManifestError),

    #[error("Failed to execute prebuilder command: {source}")]
    PrebuilderFailed { source: std::io::Error },

    #[error("Failed to download OVMF firmware: {source}")]
    DownloadOvmfFailed { source: std::io::Error },

    #[error("Failed to clone Limine repository: {source}")]
    CloneLimineFailed { source: std::io::Error },

    #[error("Failed to copy Limine config: {source}")]
    CopyLimineConfig { source: std::io::Error },

    #[error("Failed to copy Limine binary {file}: {source}")]
    CopyLimineBinary {
        file: String,
        source: std::io::Error,
    },

    #[error("Failed to copy kernel binary: {source}")]
    CopyKernel { source: std::io::Error },

    #[error("Failed to create ISO: {source}")]
    CreateIso { source: std::io::Error },

    #[error("Failed to install Limine to ISO: {source}")]
    InstallLimine { source: std::io::Error },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
