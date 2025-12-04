use std::collections::LinkedList;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::PathBuf;
use std::range::Range;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use futures::lock::Mutex;
use neptune_privacy::api::export::Network;
use neptune_privacy::application::rest_server::ExportedBlock;
use neptune_privacy::prelude::tasm_lib::prelude::Digest;
use serde::Serialize;
use sqlx::prelude::*;
use sqlx_migrator::Info;
use sqlx_migrator::Migrate;
use sqlx_migrator::Migrator;
use sqlx_migrator::Plan;

struct CreateBlockCacheMigration;

sqlx_migrator::sqlite_migration!(
    CreateBlockCacheMigration,
    "block_cache",
    "create_block_cache",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE block_cache (
            height INTEGER NOT NULL,
            hash TEXT PRIMARY KEY,
            pos INTEGER NOT NULL,
            length INTEGER NOT NULL
        )", //up
        "DROP TABLE block_cache" //down
    )]
);

struct CreateBlockCacheIndexMigration;
sqlx_migrator::sqlite_migration!(
    CreateBlockCacheIndexMigration,
    "block_cache",
    "create_block_cache_index",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE INDEX idx_block_cache_height ON block_cache (height)", //up
        "DROP INDEX idx_block_cache_height"                            //down
    )]
);

pub struct PersistBlockCache {
    pool: sqlx::SqlitePool,
    block_dir: PathBuf,
    memory_cache: MemoryBlockCache,
}

impl PersistBlockCache {
    const BLOCK_BATCH_SIZE: u64 = 2000;
    const BLOCK_FILE_EXT: &str = "block";
    pub async fn new(data_dir: &PathBuf, network: Network, cache_size: usize) -> Result<Self> {
        let block_dir = data_dir.join(format!("{}_block", network.to_string()));

        if !block_dir.exists() {
            std::fs::create_dir_all(&block_dir)
                .map_err(|err| anyhow::anyhow!("Could not create block directory: {err}"))?;
        }
        let database = block_dir.join("block.db");
        let pool = {
            let options = sqlx::sqlite::SqliteConnectOptions::new()
                .filename(database)
                .create_if_missing(true);

            sqlx::SqlitePool::connect_with(options)
                .await
                .map_err(|err| anyhow::anyhow!("Could not connect to database: {err}"))?
        };

        let mut migrator = Migrator::default();
        // Adding migration can fail if another migration with same app and name and different values gets added
        // Adding migrations add its parents, replaces and not before as well
        migrator.add_migration(Box::new(CreateBlockCacheMigration))?;
        migrator.add_migration(Box::new(CreateBlockCacheIndexMigration))?;

        let mut conn = pool.acquire().await?;
        // use apply all to apply all pending migration
        migrator.run(&mut *conn, &Plan::apply_all()).await?;

        Ok(PersistBlockCache {
            pool,
            block_dir,
            memory_cache: MemoryBlockCache::new(cache_size),
        })
    }

    fn block_path(&self, height: u64) -> PathBuf {
        let file_idx = height / Self::BLOCK_BATCH_SIZE;
        self.block_dir
            .join(format!("{file_idx}.{}", Self::BLOCK_FILE_EXT))
    }

    async fn find_block(&self, height: u64, hash: &str) -> Result<Option<(i64, i64)>> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query("SELECT pos, length FROM block_cache WHERE height = ? AND hash = ?")
            .bind(height as i64)
            .bind(hash)
            .fetch_one(&mut *conn)
            .await;

        match row {
            Ok(row) => {
                let pos = row.get::<i64, _>("pos");
                let length = row.get::<i64, _>("length");
                Ok(Some((pos, length)))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err)?,
        }
    }

    async fn find_block_by_height(&self, height: u64) -> Result<Option<(i64, i64)>> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query("SELECT pos, length FROM block_cache WHERE height = ?")
            .bind(height as i64)
            .fetch_one(&mut *conn)
            .await;

        match row {
            Ok(row) => {
                let pos = row.get::<i64, _>("pos");
                let length = row.get::<i64, _>("length");
                Ok(Some((pos, length)))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err)?,
        }
    }

    async fn find_block_by_digest(&self, digest: Digest) -> Result<Option<(i64, i64, i64)>> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query("SELECT height, pos, length FROM block_cache WHERE hash = ?")
            .bind(&digest.to_hex())
            .fetch_one(&mut *conn)
            .await;

        match row {
            Ok(row) => {
                let height = row.get::<i64, _>("height");
                let pos = row.get::<i64, _>("pos");
                let length = row.get::<i64, _>("length");
                Ok(Some((height, pos, length)))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err)?,
        }
    }

    async fn read_block_by_pos(&self, height: u64, pos: i64, length: i64) -> Result<ExportedBlock> {
        let block_file = self.block_path(height);
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(&block_file)
            .map_err(|err| anyhow::anyhow!("Could not open block file: {err}"))?;
        file.seek(std::io::SeekFrom::Start(pos as u64))
            .map_err(|err| anyhow::anyhow!("Could not seek to block position: {err}"))?;
        let mut buffer = vec![0u8; length as usize];
        file.read_exact(&mut buffer)
            .map_err(|err| anyhow::anyhow!("Could not read block from file: {err}"))?;
        let block: ExportedBlock = decode_block(&buffer)
            .map_err(|err| anyhow::anyhow!("Could not deserialize block: {err}"))?;
        Ok(block)
    }

    pub async fn delete_block_file(file: PathBuf) -> Result<()> {
        let network_dir = file.parent().context("No parent directory")?;
        let tmp = network_dir.file_name().unwrap().to_string_lossy();
        let network_str = tmp.split('_').nth(0).context("No network")?;
        let network = Network::from_str(network_str).map_err(|_| anyhow!("Invalid network"))?;
        let tmp = file.file_name().context("No file name")?.to_string_lossy();
        let batch_num_str = tmp.split('.').nth(0).context("No batch number")?;
        let batch_num = batch_num_str
            .parse::<u64>()
            .context("Invalid batch number")?;
        let block_range = Range {
            start: (batch_num * Self::BLOCK_BATCH_SIZE) as i64,
            end: ((batch_num + 1) * Self::BLOCK_BATCH_SIZE) as i64,
        };
        let cache = Self::new(&network_dir.into(), network, 1).await?;

        tokio::fs::remove_file(file)
            .await
            .map_err(|err| anyhow::anyhow!("Could not delete block file: {err}"))?;

        cache.delete_block_range(block_range).await?;
        Ok(())
    }

    async fn delete_block_range(&self, range: Range<i64>) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("DELETE FROM block_cache WHERE height >= ? AND height < ?")
            .bind(range.start)
            .bind(range.end)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    pub fn list_cache_files(data_dir: &PathBuf) -> Result<Vec<BlockCacheFile>> {
        let mut files = vec![];
        let dirs = [
            Network::Main,
            Network::RegTest,
            Network::TestnetMock,
            Network::Testnet(0),
        ]
        .map(|network| {
            (
                network.to_string(),
                format!("{}_blocks", network.to_string()),
            )
        });

        for (network, dir) in dirs {
            let path = PathBuf::from(data_dir).join(dir);
            if path.exists() {
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    if entry.file_type()?.is_file() {
                        let tmp = entry.file_name();
                        let file_name = tmp.to_string_lossy();
                        if file_name.ends_with(".block") {
                            let batch_num = file_name
                                .split('.')
                                .nth(0)
                                .context("No batch number")?
                                .parse::<u64>()
                                .context("Invalid batch number")?;
                            files.push(BlockCacheFile {
                                path: entry.path().to_string_lossy().to_string(),
                                network: network.clone(),
                                range: (
                                    (batch_num * Self::BLOCK_BATCH_SIZE) as i64,
                                    ((batch_num + 1) * Self::BLOCK_BATCH_SIZE) as i64,
                                ),
                                size: entry.metadata().unwrap().len(),
                            });
                        }
                    }
                }
            }
        }

        return Ok(files);
    }
}

#[derive(Debug, Serialize)]
pub struct BlockCacheFile {
    pub path: String,
    pub network: String,
    pub range: (i64, i64),
    pub size: u64,
}

impl BlockCache for PersistBlockCache {
    async fn add_block(&self, block: ExportedBlock) -> Result<()> {
        let height: u64 = block.kernel.header.height.into();
        let hash = block.hash().to_hex();

        if self.find_block(height, &hash).await?.is_some() {
            return Ok(()); // Block already exists
        }

        let block_serialized = encode_block(&block)?;
        let block_path = self.block_path(height);

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&block_path)
            .map_err(|err| anyhow::anyhow!("Could not open block file: {err}"))?;
        let pos = file
            .seek(std::io::SeekFrom::End(0))
            .map_err(|err| anyhow::anyhow!("Could not seek to end of file: {err}"))?
            as i64;
        let length = block_serialized.len() as i64;
        file.write_all(&block_serialized)
            .map_err(|err| anyhow::anyhow!("Could not write block to file: {err}"))?;
        file.flush()
            .map_err(|err| anyhow::anyhow!("Could not flush block file: {err}"))?;

        let mut conn = self.pool.acquire().await?;
        sqlx::query("INSERT INTO block_cache (height, hash, pos, length) VALUES (?, ?, ?, ?)")
            .bind(height as i64)
            .bind(&hash)
            .bind(pos)
            .bind(length)
            .execute(&mut *conn)
            .await
            .map_err(|err| anyhow::anyhow!("Could not insert block into database: {err}"))?;

        Ok(())
    }
    async fn add_blocks<T: Iterator<Item = ExportedBlock>>(&self, blocks: T) -> Result<()> {
        for block in blocks {
            self.add_block(block).await?;
        }
        Ok(())
    }

    async fn add_blocks_temp<T: Iterator<Item = ExportedBlock>>(&self, blocks: T) -> Result<()> {
        self.memory_cache.add_blocks_temp(blocks).await
    }

    async fn delete_block_by_start_height(&self, start_height: u64) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("DELETE FROM block_cache WHERE height >= ?")
            .bind(start_height as i64)
            .execute(&mut *conn)
            .await
            .map_err(|err| anyhow::anyhow!("Could not delete blocks from database: {err}"))?;

        self.memory_cache
            .delete_block_by_start_height(start_height)
            .await?;
        Ok(())
    }

    async fn has_block_by_height(&self, height: u64) -> Result<bool> {
        self.find_block_by_height(height)
            .await
            .map(|opt| opt.is_some())
    }

    async fn get_block_by_height(&self, height: u64) -> Result<Option<ExportedBlock>> {
        if let Some(block) = self.memory_cache.get_block_by_height(height).await? {
            return Ok(Some(block));
        }

        let (pos, length) = match self.find_block_by_height(height).await? {
            Some(block_info) => block_info,
            None => return Ok(None),
        };

        Ok(Some(self.read_block_by_pos(height, pos, length).await?))
    }

    async fn get_block_by_digest(&self, digest: Digest) -> Result<Option<ExportedBlock>> {
        if let Some(block) = self.memory_cache.get_block_by_digest(digest).await? {
            return Ok(Some(block));
        }

        let (height, pos, length) = match self.find_block_by_digest(digest).await? {
            Some(block_info) => block_info,
            None => return Ok(None),
        };

        Ok(Some(
            self.read_block_by_pos(height as u64, pos, length).await?,
        ))
    }
}

#[derive(Debug)]
pub struct MemoryBlockCache {
    cache: Mutex<LinkedList<ExportedBlock>>,
    size: usize,
}

impl MemoryBlockCache {
    fn new(size: usize) -> Self {
        Self {
            cache: Mutex::new(LinkedList::new()),
            size,
        }
    }
}

impl BlockCache for MemoryBlockCache {
    async fn add_block(&self, block: ExportedBlock) -> Result<()> {
        let mut cache = self.cache.lock().await;
        cache.push_back(block.to_owned());
        if cache.len() > self.size {
            cache.pop_front();
        }
        Ok(())
    }

    async fn add_blocks_temp<T: Iterator<Item = ExportedBlock>>(&self, blocks: T) -> Result<()> {
        self.add_blocks(blocks).await
    }

    async fn add_blocks<T: Iterator<Item = ExportedBlock>>(&self, blocks: T) -> Result<()> {
        let mut cache = self.cache.lock().await;
        for block in blocks {
            cache.push_back(block);
            if cache.len() > self.size {
                cache.pop_front();
            }
        }
        Ok(())
    }

    async fn has_block_by_height(&self, height: u64) -> Result<bool> {
        let cache = self.cache.lock().await;
        Ok(cache
            .iter()
            .any(|b| b.kernel.header.height == height.into()))
    }

    async fn get_block_by_height(&self, height: u64) -> Result<Option<ExportedBlock>> {
        let cache = self.cache.lock().await;
        if let Some(block) = cache
            .iter()
            .find(|b| b.kernel.header.height == height.into())
        {
            return Ok(Some(block.clone()));
        };

        Ok(None)
    }

    async fn get_block_by_digest(&self, digest: Digest) -> Result<Option<ExportedBlock>> {
        let cache = self.cache.lock().await;
        if let Some(block) = cache.iter().find(|b| b.hash() == digest) {
            return Ok(Some(block.clone()));
        };

        Ok(None)
    }

    async fn delete_block_by_start_height(&self, start_height: u64) -> Result<()> {
        let mut cache = self.cache.lock().await;
        cache.retain(|b| b.kernel.header.height < start_height.into());
        Ok(())
    }
}

#[enum_dispatch(BlockCacheImpl)]
pub(super) trait BlockCache {
    async fn add_block(&self, block: ExportedBlock) -> Result<()>;
    async fn add_blocks<T: Iterator<Item = ExportedBlock>>(&self, blocks: T) -> Result<()>;
    async fn add_blocks_temp<T: Iterator<Item = ExportedBlock>>(&self, blocks: T) -> Result<()>;
    async fn has_block_by_height(&self, height: u64) -> Result<bool>;
    async fn get_block_by_height(&self, height: u64) -> Result<Option<ExportedBlock>>;
    async fn get_block_by_digest(&self, digest: Digest) -> Result<Option<ExportedBlock>>;
    async fn delete_block_by_start_height(&self, start_height: u64) -> Result<()>;
}

#[enum_dispatch]
pub enum BlockCacheImpl {
    Persist(PersistBlockCache),
    Memory(MemoryBlockCache),
}

impl BlockCacheImpl {
    pub fn new_memory(cache_size: usize) -> Self {
        BlockCacheImpl::Memory(MemoryBlockCache::new(cache_size))
    }

    pub async fn new_persist(
        data_dir: &PathBuf,
        network: Network,
        cache_size: usize,
    ) -> Result<Self> {
        Ok(BlockCacheImpl::Persist(
            PersistBlockCache::new(data_dir, network, cache_size).await?,
        ))
    }

    pub fn is_persist(&self) -> bool {
        matches!(self, BlockCacheImpl::Persist(_))
    }
}

fn encode_block(block: &ExportedBlock) -> Result<Vec<u8>> {
    let block_serialized = bincode::serialize(block)
        .map_err(|err| anyhow::anyhow!("Could not serialize block: {err}"))?;
    let mut buffer = vec![];
    let mut encoder = zstd::Encoder::with_dictionary(&mut buffer, 17, ZSTD_DICT)?;
    encoder.write_all(&block_serialized)?;
    encoder.finish()?;
    Ok(buffer)
}

fn decode_block(block_bytes: &[u8]) -> Result<ExportedBlock> {
    let mut decoder = zstd::Decoder::with_dictionary(block_bytes, ZSTD_DICT)?;
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(|err| anyhow::anyhow!("Could not decode block: {err}"))?;
    Ok(bincode::deserialize(&decoded)?)
}

static ZSTD_DICT: &'static [u8] = &[
    55, 164, 48, 236, 7, 34, 148, 5, 9, 16, 16, 223, 48, 51, 51, 179, 119, 10, 51, 241, 120, 60,
    30, 143, 199, 227, 241, 120, 60, 207, 243, 188, 247, 212, 66, 65, 65, 65, 65, 65, 65, 65, 65,
    65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 161, 80, 40, 20, 10, 133,
    66, 161, 80, 40, 20, 10, 133, 162, 40, 138, 162, 40, 74, 41, 125, 116, 225, 225, 225, 225, 225,
    225, 225, 225, 225, 225, 225, 225, 225, 225, 225, 225, 225, 225, 225, 241, 120, 60, 30, 143,
    199, 227, 241, 120, 158, 231, 121, 239, 1, 1, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 4, 0, 0, 0, 0, 0, 0, 0, 40, 183, 82, 174, 113, 145, 140, 118, 18, 202, 97, 91, 116, 107,
    176, 146, 229, 43, 126, 154, 64, 206, 227, 164, 103, 80, 43, 109, 166, 211, 6, 35, 77, 89, 94,
    179, 196, 243, 64, 239, 116, 117, 72, 92, 42, 167, 145, 98, 97, 82, 140, 37, 209, 235, 115,
    181, 191, 52, 170, 80, 210, 84, 19, 22, 244, 90, 171, 244, 109, 27, 52, 131, 96, 144, 234, 191,
    150, 88, 133, 11, 222, 206, 146, 184, 20, 10, 66, 160, 209, 127, 19, 107, 37, 23, 201, 84, 9,
    42, 28, 132, 122, 151, 168, 193, 140, 68, 139, 206, 164, 4, 33, 109, 220, 75, 200, 125, 97,
    223, 50, 145, 168, 91, 207, 36, 61, 46, 10, 11, 208, 143, 247, 23, 57, 186, 140, 177, 73, 49,
    78, 179, 136, 159, 200, 98, 47, 223, 57, 208, 195, 119, 209, 28, 216, 190, 131, 129, 93, 62,
    15, 17, 1, 0, 0, 0, 0, 0, 0, 0, 192, 248, 203, 199, 58, 132, 74, 182, 195, 88, 109, 136, 145,
    226, 155, 103, 122, 58, 160, 143, 37, 249, 174, 192, 248, 84, 167, 43, 242, 226, 248, 76, 42,
    72, 201, 221, 27, 190, 10, 102, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 139, 141, 30, 163, 2, 191,
    163, 5, 176, 77, 242, 170, 5, 137, 57, 61, 100, 88, 92, 113, 253, 17, 58, 60, 19, 87, 103, 132,
    224, 58, 192, 199, 198, 45, 120, 72, 201, 88, 233, 70, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 33, 56, 0, 0, 0, 0, 0, 0, 231, 248, 139, 226, 133, 117, 234, 96, 254, 234,
    212, 111, 171, 160, 194, 109, 175, 182, 105, 126, 24, 79, 142, 67, 136, 192, 125, 78, 188, 17,
    196, 167, 26, 162, 0, 0, 0, 0, 0, 0, 47, 0, 182, 229, 150, 1, 0, 0, 34, 32, 117, 171, 29, 8,
    209, 142, 162, 15, 245, 103, 29, 40, 168, 217, 142, 252, 184, 78, 231, 34, 0, 169, 45, 26, 151,
    237, 71, 97, 195, 89, 205, 11, 12, 209, 121, 113, 198, 18, 0, 46, 110, 37, 126, 35, 111, 39, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 138, 156, 50, 163, 123, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 240, 2, 227, 190, 33, 203, 70, 102, 114, 43, 173, 19, 74, 71, 126, 253,
    172, 8, 78, 175, 208, 203, 201, 195, 217, 125, 199, 41, 119, 214, 102, 44, 2, 49, 119, 55, 136,
    255, 185, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 128, 212, 219, 233, 140, 160, 57, 89, 62, 25, 0, 0, 1, 0, 0, 0, 0, 128, 212, 219, 233, 140,
    160, 57, 89, 62, 25, 0, 0, 47, 0, 182, 229, 150, 1, 0, 0, 174, 85, 124, 133, 196, 38, 87, 193,
    211, 15, 255, 94, 175, 161, 183, 78, 231, 92, 96, 179, 113, 66, 176, 67, 200, 135, 142, 145,
    106, 42, 160, 81, 39, 82, 116, 184, 121, 180, 29, 168, 1, 26, 252, 0, 0, 0, 0, 0, 0, 9, 0, 0,
    0, 0, 0, 0, 0, 185, 27, 242, 236, 78, 5, 7, 187, 144, 28, 209, 189, 221, 36, 84, 86, 196, 25,
    247, 227, 121, 28, 123, 173, 204, 6, 200, 123, 136, 228, 176, 67, 93, 229, 171, 139, 250, 73,
    202, 193, 31, 201, 3, 186, 237, 197, 221, 187, 73, 92, 226, 102, 239, 237, 182, 81, 255, 213,
    238, 25, 233, 112, 192, 182, 142, 131, 71, 17, 150, 53, 47, 90, 101, 188, 119, 214, 214, 186,
    149, 67, 50, 135, 196, 80, 95, 100, 198, 114, 1, 41, 77, 223, 128, 27, 191, 204, 151, 53, 150,
    42, 249, 152, 63, 0, 159, 54, 241, 32, 119, 116, 90, 169, 49, 238, 54, 40, 248, 187, 90, 174,
    7, 154, 179, 50, 196, 84, 250, 99, 178, 100, 222, 44, 112, 13, 154, 29, 126, 148, 228, 171,
    192, 194, 159, 67, 251, 179, 19, 41, 108, 163, 135, 9, 94, 211, 78, 38, 4, 115, 18, 216, 5, 79,
    214, 82, 232, 230, 206, 191, 50, 39, 174, 134, 28, 215, 227, 112, 13, 163, 26, 247, 159, 199,
    182, 39, 139, 103, 62, 52, 204, 101, 237, 77, 232, 25, 167, 2, 54, 144, 200, 104, 250, 17, 139,
    17, 187, 57, 252, 177, 0, 45, 73, 81, 234, 119, 75, 129, 243, 40, 81, 209, 22, 25, 243, 42, 21,
    118, 198, 167, 93, 2, 156, 221, 181, 102, 186, 240, 30, 233, 202, 1, 118, 109, 13, 166, 182,
    38, 113, 247, 233, 2, 13, 212, 135, 69, 76, 218, 229, 72, 220, 79, 97, 55, 51, 167, 112, 161,
    29, 98, 210, 45, 40, 213, 210, 74, 85, 232, 163, 192, 239, 74, 193, 137, 66, 36, 41, 237, 27,
    143, 239, 178, 92, 127, 162, 165, 190, 102, 124, 77, 152, 250, 35,
];
