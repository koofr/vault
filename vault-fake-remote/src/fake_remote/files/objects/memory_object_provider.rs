use std::{collections::HashMap, io::SeekFrom, ops::RangeInclusive, pin::Pin, sync::Mutex};

use async_trait::async_trait;
use futures::{io::Cursor, AsyncRead, AsyncReadExt, AsyncSeekExt};

use super::object_provider::{ObjectProvider, ObjectProviderError};

pub struct MemoryObjectProvider {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemoryObjectProvider {
    pub fn new() -> Self {
        Self {
            objects: Default::default(),
        }
    }
}

impl std::fmt::Debug for MemoryObjectProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryObjectProvider").finish()
    }
}

#[async_trait]
impl ObjectProvider for MemoryObjectProvider {
    async fn get(
        &self,
        object_id: String,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<Pin<Box<dyn AsyncRead + Send + Sync + 'static>>, ObjectProviderError> {
        let buf = self
            .objects
            .lock()
            .unwrap()
            .get(&object_id)
            .cloned()
            .ok_or_else(|| {
                ObjectProviderError::IOError(
                    std::io::Error::from(std::io::ErrorKind::NotFound).into(),
                )
            })?;

        let mut reader = Cursor::new(buf);

        match range {
            Some(range) => {
                reader.seek(SeekFrom::Start(*range.start())).await?;

                Ok(Box::pin(reader.take(range.end() - range.start() + 1)))
            }
            None => Ok(Box::pin(reader)),
        }
    }

    async fn put(
        &self,
        object_id: String,
        mut reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
    ) -> Result<u64, ObjectProviderError> {
        let mut buf = Vec::new();

        reader.read_to_end(&mut buf).await?;

        let size = buf.len() as u64;

        self.objects.lock().unwrap().insert(object_id, buf);

        Ok(size)
    }

    async fn delete(&self, object_id: String) -> Result<(), ObjectProviderError> {
        let object = self.objects.lock().unwrap().remove(&object_id);

        match object {
            Some(_) => Ok(()),
            None => Err(ObjectProviderError::IOError(
                std::io::Error::from(std::io::ErrorKind::NotFound).into(),
            )),
        }
    }
}
