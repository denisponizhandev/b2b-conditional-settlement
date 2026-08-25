use std::env;

pub const DEFAULT_MAX_CHUNK_BLOCKS: u64 = 9_999;

pub struct IndexerConfig {
    pub start_block: u64,
    pub poll_interval_secs: Option<u64>,
    pub max_chunk_blocks: u64,
    pub chunk_delay_ms: u64,
    pub chunk_max_retries: u32
}

impl IndexerConfig {
    pub fn from_env() -> Result<Self, String> {
        let start_block = env::var("INDEXER_START_BLOCK")
            .map_err(|_| "INDEXER_START_BLOCK is not set".to_string())?
            .parse::<u64>()
            .map_err(|_| "INDEXER_START_BLOCK must be u64".to_string())?;
        
        let poll_interval_secs = match env::var("INDEXER_POLL_INTERVAL_SECS") {
            Ok(raw) => Some(
                raw.parse::<u64>() 
                    .map_err(|_| "INDEXER_POLL_INTERVAL_SECS must be u64".to_string())?,
            ),
            Err(_) => None
        };

        let max_chunk_blocks = env::var("INDEXER_MAX_CHUNK_BLOCKS")
            .ok()
            .map(|raw| {
                raw.parse::<u64>()
                    .map_err(|_| "INDEXER_MAX_CHUNK_BLOCKS must be u64".to_string())
            })
            .transpose()?
            .unwrap_or(DEFAULT_MAX_CHUNK_BLOCKS);
        
        let chunk_delay_ms = env::var("INDEXER_CHUNK_DELAY_MS")
            .ok()
            .map(|raw| {
                raw.parse::<u64>()
                    .map_err(|_| "INDEXER_CHUNK_DELAY_MS must be u64".to_string())
            })
            .transpose()?
            .unwrap_or(500);

        let chunk_max_retries = env::var("INDEXER_CHUNK_MAX_RETRIES")
            .ok()
            .map(|raw| {
                raw.parse::<u32>()
                    .map_err(|_| "INDEXER_CHUNK_MAX_RETRIES must be u32".to_string())
            })
            .transpose()?
            .unwrap_or(5);
    

        Ok(Self {
            start_block,
            poll_interval_secs,
            max_chunk_blocks,
            chunk_delay_ms,
            chunk_max_retries
        })
    }
}