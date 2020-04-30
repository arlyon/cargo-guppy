// Copyright (c) The cargo-guppy Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::graph::PackageGraph;
use crate::Error;
use cargo_metadata::CargoOpt;
use std::path::Path;

/// A builder for configuring `cargo metadata` invocations.
///
/// ## Examples
///
/// Build a `PackageGraph` for the Cargo workspace in the current directory:
///
/// ```rust
/// use guppy::MetadataCommand;
/// use guppy::graph::PackageGraph;
///
/// let mut cmd = MetadataCommand::new();
/// let package_graph = PackageGraph::from_command(&mut cmd);
/// ```
#[derive(Clone, Debug, Default)]
pub struct MetadataCommand {
    inner: cargo_metadata::MetadataCommand,
}

impl MetadataCommand {
    /// Creates a default `cargo metadata` command builder.
    ///
    /// By default, this will look for `Cargo.toml` in the ancestors of this process's current
    /// directory.
    pub fn new() -> Self {
        let mut inner = cargo_metadata::MetadataCommand::new();
        // Always use --all-features so that we get a full view of the graph.
        inner.features(CargoOpt::AllFeatures);
        Self { inner }
    }

    /// Sets the path to the `cargo` executable.
    ///
    /// If unset, this will use the `$CARGO` environment variable, or else `cargo` from `$PATH`.
    pub fn cargo_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cargo_path(path);
        self
    }

    /// Sets the path to `Cargo.toml`.
    ///
    /// By default, this will look for `Cargo.toml` in the ancestors of the current directory. Note
    /// that this doesn't need to be the root `Cargo.toml` in a workspace -- any member of the
    /// workspace is fine.
    pub fn manifest_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.manifest_path(path);
        self
    }

    /// Sets the current directory of the `cargo metadata` process.
    ///
    /// By default, the current directory will be inherited from this process.
    pub fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.current_dir(path);
        self
    }

    // *Do not* implement no_deps or features.

    /// Arbitrary flags to pass to `cargo metadata`. These will be added to the end of the
    /// command invocation.
    ///
    /// Note that `guppy` internally:
    /// * passes in `--all-features`, so that `guppy` has a full view of the dependency graph.
    /// * does not pass in `--no-deps`, so that `guppy` knows about non-workspace dependencies.
    ///
    /// Attempting to override either of those options may lead to unexpected results.
    pub fn other_options(&mut self, options: impl AsRef<[String]>) -> &mut Self {
        self.inner.other_options(options);
        self
    }

    /// Runs the configured `cargo metadata` and returns a parsed `CargoMetadata`.
    pub fn exec(&mut self) -> Result<CargoMetadata, Error> {
        let inner = self.inner.exec().map_err(Error::command_error)?;
        Ok(CargoMetadata(inner))
    }

    /// Runs the configured `cargo metadata` and returns a parsed `PackageGraph`.
    pub fn build_graph(&mut self) -> Result<PackageGraph, Error> {
        let metadata = self.exec()?;
        metadata.build_graph()
    }
}

/// A parsed `Cargo` metadata returned by a `MetadataCommand`.
///
/// This is an intermediate, opaque struct which may be generated either through
/// `MetadataCommand::to_metadata` or through deserializing JSON representing `cargo metadata`. To
/// analyze Cargo metadata by building a `PackageGraph`
/// from it, call the `into_package_graph` method.
#[derive(Clone, Debug)]
pub struct CargoMetadata(pub(crate) cargo_metadata::Metadata);

impl CargoMetadata {
    /// Parses this JSON blob into a `Metadata`.
    pub fn parse_json(json: impl AsRef<str>) -> Result<Self, Error> {
        let inner = serde_json::from_str(json.as_ref()).map_err(Error::MetadataParseError)?;
        Ok(Self(inner))
    }

    /// Builds a `PackageGraph` out of this `Metadata`.
    pub fn build_graph(self) -> Result<PackageGraph, Error> {
        PackageGraph::from_metadata(self)
    }
}
