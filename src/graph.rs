// Copyright (c) The cargo-guppy Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::errors::Error;
use cargo_metadata::{Metadata, MetadataCommand, NodeDep, Package, PackageId};
use petgraph::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct PackageGraph {
    packages: HashMap<PackageId, PackageData>,
}

#[derive(Clone, Debug)]
pub struct PackageData {
    node_idx: NodeIndex<u32>,
    in_workspace: bool,
    package: Package,
    resolved_deps: Vec<NodeDep>,
    resolved_features: Vec<String>,
}

impl PackageGraph {
    pub fn from_command(command: &mut MetadataCommand) -> Result<Self, Error> {
        Self::new(command.exec().map_err(Error::CommandError)?)
    }

    pub fn new(metadata: Metadata) -> Result<Self, Error> {
        let resolve = metadata.resolve.ok_or_else(|| {
            Error::DepGraphError(
                "no 'resolve' entries found: ensure you don't have no_deps set".into(),
            )
        })?;

        let mut resolve_data: HashMap<_, _> = resolve
            .nodes
            .into_iter()
            .map(|node| (node.id, (node.deps, node.features)))
            .collect();

        let workspace_members = metadata
            .workspace_members
            .into_iter()
            .collect::<HashSet<_>>();

        let mut graph: Graph<_, ()> = Graph::new();

        let packages = metadata
            .packages
            .into_iter()
            .map(|package| {
                let node_idx = graph.add_node(package.id.clone());
                let in_workspace = workspace_members.contains(&package.id);
                let (resolved_deps, resolved_features) = match resolve_data.remove(&package.id) {
                    Some(resolve_data) => resolve_data,
                    None => {
                        return Err(Error::DepGraphError(format!(
                            "no resolve data found for package '{}'",
                            package.id
                        )));
                    }
                };
                Ok((
                    package.id.clone(),
                    PackageData {
                        node_idx,
                        in_workspace,
                        package,
                        resolved_deps,
                        resolved_features,
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, Error>>()?;

        for (id, data) in &packages {
            // TODO: use the resolved deps to figure out what deps are being used
            // see https://github.com/sfackler/cargo-tree/blob/master/src/main.rs#L388

        }

        Ok(Self { packages })
    }
}