use std::cmp::Ordering;
use std::collections::LinkedList;
use std::io::Read;
use std::io::SeekFrom;
use std::io::Write;
use std::path::PathBuf;
use std::range::Range;
use std::sync::Arc;

use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use neptune_privacy::api::export::Network;
use neptune_privacy::application::rest_server::ExportedBlock;
use neptune_privacy::prelude::tasm_lib::prelude::Digest;
use serde::Deserialize;
use serde::Serialize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tracing::*;

use crate::rpc_client;
use crate::wallet::block_cache::BlockCache;
use crate::wallet::block_cache::BlockCacheImpl;

#[derive(Clone)]
pub struct FakeArchivalState {
    block_cache: Arc<BlockCacheImpl>,
    snapshot_reader: Arc<Option<SnapshotReader>>,
    network: Network,
}

impl FakeArchivalState {
    pub fn new(
        cache: BlockCacheImpl,
        network: Network,
        snapshot_reader: Option<SnapshotReader>,
    ) -> Self {
        Self {
            block_cache: Arc::new(cache),
            snapshot_reader: Arc::new(snapshot_reader),
            network,
        }
    }

    pub async fn prepare(&self, height: u64, batch_size: u64) -> Result<()> {
        if self.block_cache.is_persist() && self.block_cache.has_block_by_height(height).await? {
            debug!("Block {height} already in cache, skipping request");
            return Ok(());
        }

        if let Some(blocks) = self.read_block_from_snapshot(height, batch_size).await {
            // add to temp cache as it is already in snapshot
            self.block_cache.add_blocks_temp(blocks.into_iter()).await?;
            return Ok(());
        }

        let blocks = rpc_client::node_rpc_client()
            .request_block_by_height_range(height, batch_size)
            .await?;
        self.block_cache.add_blocks(blocks.into_iter()).await?;
        Ok(())
    }

    async fn read_block_from_snapshot(
        &self,
        height: u64,
        batch_size: u64,
    ) -> Option<Vec<ExportedBlock>> {
        if let Some(reader) = self.snapshot_reader.as_ref() {
            return reader
                .read_blocks(self.network, (height..height + batch_size).into())
                .await;
        }
        None
    }

    pub async fn get_block_by_height(&self, height: u64) -> Result<Option<ExportedBlock>> {
        if let Some(block) = self.block_cache.get_block_by_height(height).await? {
            return Ok(Some(block.clone()));
        }

        debug!("get_block_by_height: requesting block {height} from rest server");
        let result = rpc_client::node_rpc_client().request_block(height).await?;
        if self.block_cache.is_persist() {
            if let Some(block) = &result {
                self.block_cache.add_block(block.clone()).await?;
            }
        }

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn get_block_by_digest(&self, digest: Digest) -> Result<Option<ExportedBlock>> {
        if let Some(block) = self.block_cache.get_block_by_digest(digest).await? {
            return Ok(Some(block.clone()));
        }

        debug!(
            "get_block_by_digest: requesting block {} from rest server",
            digest.to_hex()
        );
        Ok(rpc_client::node_rpc_client()
            .request_block_by_digest(&digest.to_hex())
            .await?)
    }

    pub async fn reset_to_height(&self, height: u64) -> Result<()> {
        self.block_cache
            .delete_block_by_start_height(height + 1)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockPosition {
    pub pos: u64,
    pub size: u64,
}

impl BlockPosition {
    const SIZE: usize = 16;

    fn to_le_bytes(&self) -> [u8; Self::SIZE] {
        let pos = self.pos.to_le_bytes();
        let size = self.size.to_le_bytes();
        [
            pos[0], pos[1], pos[2], pos[3], pos[4], pos[5], pos[6], pos[7], size[0], size[1],
            size[2], size[3], size[4], size[5], size[6], size[7],
        ]
    }

    fn from_le_bytes(bytes: [u8; Self::SIZE]) -> Self {
        let pos = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let size = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        Self { pos, size }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct SnapshotNetwork(Network);
impl SnapshotNetwork {
    fn to_byte(&self) -> u8 {
        match self.0 {
            Network::Main => 0,
            Network::TestnetMock => 1,
            Network::RegTest => 2,
            Network::Testnet(i) => i as u8 + 3,
            _ => todo!(),
        }
    }

    fn try_from_byte(bytes: u8) -> Result<Self> {
        Ok(Self(match bytes {
            0 => Network::Main,
            1 => Network::TestnetMock,
            2 => Network::RegTest,
            3 => Network::Testnet(0),
            _ => bail!("Invalid network"),
        }))
    }
}

impl Ord for SnapshotNetwork {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_byte().cmp(&other.to_byte())
    }
}

impl PartialOrd for SnapshotNetwork {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct SnapshotMetadata {
    pub range: Range<u64>,
    pub network: SnapshotNetwork,
    pub path: PathBuf,
}

impl SnapshotMetadata {
    async fn read_from_file(path: PathBuf) -> Result<Self> {
        let mut file = File::open(&path).await?;
        let mut buf = [0u8; 17];
        if file.read_exact(&mut buf).await.is_ok() {
            let start = u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]);
            let end = u64::from_le_bytes([
                buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
            ]);
            let network = SnapshotNetwork::try_from_byte(buf[16])?;
            return Ok(Self {
                path,
                range: (start..end).into(),
                network,
            });
        }
        bail!("Failed to read snapshot metadata")
    }

    async fn read_dict(&self, file: &mut File) -> Result<Vec<u8>> {
        let dict_pos = 17u64 + ((self.range.end - self.range.start) * BlockPosition::SIZE as u64);
        file.seek(SeekFrom::Start(dict_pos)).await?;
        let dict_size = file.read_u64_le().await?;
        info!("dict_size: {}", dict_size);
        let mut dict = vec![0u8; dict_size as usize];
        file.read_exact(&mut dict).await?;
        Ok(dict)
    }

    async fn write_metadata_and_place_holder(&self, file: &mut File) -> Result<()> {
        let start = self.range.start.to_le_bytes();
        let end = self.range.end.to_le_bytes();
        file.write_all(&start).await?;
        file.write_all(&end).await?;
        file.write_u8(self.network.to_byte()).await?;

        let placeholder = vec![0; BlockPosition::SIZE];
        for _ in self.range {
            file.write_all(&placeholder).await?;
        }

        Ok(())
    }

    async fn update_block_position(
        &self,
        file: &mut File,
        blocks: Vec<BlockPosition>,
    ) -> Result<()> {
        let seek_pos = file.seek(SeekFrom::Start(17)).await?; // Skip the metadata size u64
        ensure!(seek_pos == 17, "Seek failed to move to metadata position");
        for block in blocks.iter() {
            file.write_all(&block.to_le_bytes()).await?;
        }
        Ok(())
    }

    fn contains_block(&self, range: Range<u64>) -> bool {
        self.range.start <= range.start && self.range.end >= range.end
    }
}

// [size][vec<block_pos>][0][vec[block]]
pub async fn generate_snapshot(dir: &PathBuf, network: Network, range: Range<u64>) -> Result<()> {
    let path = dir.join(format!(
        "neptune_{}_{}-{}.{}",
        network.to_string(),
        range.start,
        range.end,
        SNAPSHOT_EXT
    ));

    let size = (range.end - range.start) as usize;

    let mut file = File::create(&path).await?;

    let metadata = SnapshotMetadata {
        path: path.clone(),
        range: range.clone(),
        network: SnapshotNetwork(network),
    };

    metadata.write_metadata_and_place_holder(&mut file).await?;

    let cache = BlockCacheImpl::new_memory(200);
    let state = FakeArchivalState::new(cache, network, None);
    state.prepare(range.start, 100).await?;
    // Write block data and update positions

    let mut blocks = Vec::with_capacity(size);
    let mut block_sizes = Vec::with_capacity(size);
    for height in range {
        if (height - 1) % 100 == 0 {
            info!("Getting snapshot block {}", height);
            state.prepare(height, 100).await?;
        }
        let block = state
            .get_block_by_height(height)
            .await?
            .context("block not found")?;

        let mut buffer = bincode::serialize(&block)?;
        block_sizes.push(buffer.len());
        blocks.append(&mut buffer);
    }
    file.sync_all().await?;

    info!("training zstd dictionary...");
    let dict = zstd::dict::from_continuous(&blocks, &block_sizes, 32 * 1024)?;
    file.write_u64_le(dict.len() as u64).await?;
    file.write_all(&dict).await?;
    file.sync_all().await?;

    info!("writing blocks...");
    let mut cursor = 0;
    let mut block_positions = Vec::with_capacity(size);
    for block_size in block_sizes {
        let pos = file.stream_position().await?;
        let mut buffer = vec![];
        let mut encoder = zstd::Encoder::with_dictionary(&mut buffer, 17, &dict)?;
        encoder.write_all(&blocks[cursor..cursor + block_size as usize])?;
        cursor += block_size as usize;
        encoder.finish()?;
        let size = buffer.len() as u64;
        file.write_all(&buffer).await?;

        block_positions.push(BlockPosition { pos, size });
    }

    // Rewrite metadata with updated block positions
    metadata
        .update_block_position(&mut file, block_positions)
        .await?;

    file.sync_all().await?;

    info!("snapshot written to {:?}", path);

    Ok(())
}

const SNAPSHOT_EXT: &str = "snapshot";
#[derive(Default, Debug)]
pub struct SnapshotReader {
    snapshots: LinkedList<SnapshotMetadata>,
}

impl SnapshotReader {
    pub async fn new(dir: &PathBuf) -> Result<Self> {
        let mut snapshots = LinkedList::new();
        let mut read_dir = tokio::fs::read_dir(dir)
            .await
            .context("list snapshot dir")?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == SNAPSHOT_EXT {
                    if let Ok(metadata) = SnapshotMetadata::read_from_file(path).await {
                        snapshots.push_back(metadata);
                    }
                }
            }
        }

        Ok(Self { snapshots })
    }

    async fn read_blocks(&self, network: Network, range: Range<u64>) -> Option<Vec<ExportedBlock>> {
        for metadata in &self.snapshots {
            if metadata.network == SnapshotNetwork(network) && metadata.contains_block(range) {
                debug!(
                    "Reading blocks {:?} from snapshot {}",
                    range,
                    metadata.path.file_name().unwrap().to_string_lossy()
                );
                match Self::read_block_from_snapshot(&metadata, range).await {
                    Ok(b) => return Some(b),
                    Err(e) => {
                        warn!(
                            "Error reading snapshot {}: {:#?}",
                            metadata.path.file_name().unwrap().to_string_lossy(),
                            e
                        );
                    }
                }
            }
        }

        None
    }

    async fn read_block_from_snapshot(
        metadata: &SnapshotMetadata,
        range: Range<u64>,
    ) -> Result<Vec<ExportedBlock>> {
        let mut file = File::open(&metadata.path)
            .await
            .context("open snapshot file")?;

        let dict = metadata
            .read_dict(&mut file)
            .await
            .context("read snapshot dict")?;
        let dict = Arc::new(dict);

        let mut blocks = Vec::with_capacity((range.end - range.start) as usize);
        for height in range {
            let block = Self::read_compressed_block(&mut file, dict.clone(), height).await?;
            blocks.push(block);
        }
        return Ok(blocks);
    }

    async fn block_position(file: &mut File, height: u64) -> Result<BlockPosition> {
        file.seek(SeekFrom::Start(0)).await?;
        let start = file.read_u64_le().await?;
        let pos = 17u64 + ((height - start) * BlockPosition::SIZE as u64);
        file.seek(SeekFrom::Start(pos)).await?;
        let mut buf = [0u8; BlockPosition::SIZE];
        let _ = file.read_exact(&mut buf).await?;
        Ok(BlockPosition::from_le_bytes(buf))
    }

    async fn read_compressed_block(
        file: &mut File,
        dict: Arc<Vec<u8>>,
        height: u64,
    ) -> Result<ExportedBlock> {
        let pos = Self::block_position(file, height)
            .await
            .context("get block position")?;
        file.seek(std::io::SeekFrom::Start(pos.pos))
            .await
            .context("seek to block position")?;

        let mut buffer = vec![0u8; pos.size as usize];
        file.read_exact(&mut buffer)
            .await
            .context("read compressed block")?;

        let block = tokio::task::spawn_blocking(move || -> Result<ExportedBlock> {
            let mut decoder =
                zstd::Decoder::with_dictionary(&buffer[..], &dict).context("create decoder")?;

            let mut decoded = Vec::new();
            decoder.read_to_end(&mut decoded).context("decode block")?;
            let block =
                bincode::deserialize::<ExportedBlock>(&decoded).context("deserialize block")?;
            Ok(block)
        })
        .await??;

        if Into::<u64>::into(block.kernel.header.height) == height {
            Ok(block)
        } else {
            bail!(
                "block height mismatch, {},{}",
                Into::<u64>::into(block.kernel.header.height),
                height
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use neptune_privacy::api::export::BlockHeight;

    use super::*;

    #[tokio::test]
    async fn test_snapshot_write_read() {
        tracing_subscriber::fmt().init();

        rpc_client::node_rpc_client().set_rest_server("http://127.0.0.1:9800".to_string());

        let temp_snapshot_file_path = PathBuf::from("./");

        generate_snapshot(&temp_snapshot_file_path, Network::Main, Range::from(5..105))
            .await
            .unwrap();

        generate_snapshot(
            &temp_snapshot_file_path,
            Network::Main,
            Range::from(200..301),
        )
        .await
        .unwrap();

        let snapshot_reader = SnapshotReader::new(&temp_snapshot_file_path).await.unwrap();
        //test FakeArchivalState
        let cache = BlockCacheImpl::new_memory(200);
        let state = FakeArchivalState::new(cache, Network::Main, Some(snapshot_reader));
        rpc_client::node_rpc_client().set_rest_server("https:/".to_string());

        let instant = Instant::now();
        state.prepare(5, 100).await.unwrap();
        info!("prepare time: {:?}", instant.elapsed());

        let block = state.get_block_by_height(104).await.unwrap().unwrap();

        assert_eq!(block.kernel.header.height, BlockHeight::from(104));

        let block1 = state.get_block_by_height(5).await.unwrap().unwrap();
        assert_eq!(block1.kernel.header.height, BlockHeight::from(5));

        let instant = Instant::now();
        state.prepare(201, 100).await.unwrap();
        info!("prepare time: {:?}", instant.elapsed());

        assert!(state.prepare(201, 101).await.is_err());

        let block300 = state.get_block_by_height(300).await.unwrap().unwrap();
        assert_eq!(block300.kernel.header.height, BlockHeight::from(300));
    }
}
