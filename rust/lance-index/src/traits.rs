// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::execution::SendableRecordBatchStream;
use lance_core::Result;

use crate::{optimize::OptimizeOptions, IndexParams, IndexType};
use lance_table::format::Index;
use uuid::Uuid;

// Extends Lance Dataset with secondary index.
#[async_trait]
pub trait DatasetIndexExt {
    /// Create indices on columns.
    ///
    /// Upon finish, a new dataset version is generated.
    ///
    /// Parameters:
    ///
    ///  - `columns`: the columns to build the indices on.
    ///  - `index_type`: specify [`IndexType`].
    ///  - `name`: optional index name. Must be unique in the dataset.
    ///            if not provided, it will auto-generate one.
    ///  - `params`: index parameters.
    ///  - `replace`: replace the existing index if it exists.
    async fn create_index(
        &mut self,
        columns: &[&str],
        index_type: IndexType,
        name: Option<String>,
        params: &dyn IndexParams,
        replace: bool,
    ) -> Result<()>;

    /// Drop indices by name.
    ///
    /// Upon finish, a new dataset version is generated.
    ///
    /// Parameters:
    ///
    /// - `name`: the name of the index to drop.
    async fn drop_index(&mut self, name: &str) -> Result<()>;

    /// Prewarm an index by name.
    ///
    /// This will load the index into memory and cache it.
    ///
    /// Generally, this should only be called when it is known the entire index will
    /// fit into the index cache.
    ///
    /// This is a hint that is not enforced by all indices today.  Some indices may choose
    /// to ignore this hint.
    async fn prewarm_index(&self, name: &str) -> Result<()>;

    /// Read all indices of this Dataset version.
    ///
    /// The indices are lazy loaded and cached in memory within the [`Dataset`] instance.
    /// The cache is invalidated when the dataset version (Manifest) is changed.
    async fn load_indices(&self) -> Result<Arc<Vec<Index>>>;

    /// Loads all the indies of a given UUID.
    ///
    /// Note that it is possible to have multiple indices with the same UUID,
    /// as they are the deltas of the same index.
    async fn load_index(&self, uuid: &str) -> Result<Option<Index>> {
        self.load_indices().await.map(|indices| {
            indices
                .iter()
                .find(|idx| idx.uuid.to_string() == uuid)
                .cloned()
        })
    }

    /// Loads a specific index with the given index name
    ///
    /// Returns
    /// -------
    /// - `Ok(indices)`: if the index exists, returns the index.
    /// - `Ok(vec![])`: if the index does not exist.
    /// - `Err(e)`: if there is an error loading indices.
    ///
    async fn load_indices_by_name(&self, name: &str) -> Result<Vec<Index>> {
        self.load_indices().await.map(|indices| {
            indices
                .iter()
                .filter(|idx| idx.name == name)
                .cloned()
                .collect()
        })
    }

    /// Loads a specific index with the given index name.
    async fn load_scalar_index_for_column(&self, col: &str) -> Result<Option<Index>>;

    /// Optimize indices.
    async fn optimize_indices(&mut self, options: &OptimizeOptions) -> Result<()>;

    /// Find index with a given index_name and return its serialized statistics.
    ///
    /// If the index does not exist, return Error.
    async fn index_statistics(&self, index_name: &str) -> Result<String>;

    async fn commit_existing_index(
        &mut self,
        index_name: &str,
        column: &str,
        index_id: Uuid,
    ) -> Result<()>;

    async fn read_index_partition(
        &self,
        index_name: &str,
        partition_id: usize,
        with_vector: bool,
    ) -> Result<SendableRecordBatchStream>;
}
