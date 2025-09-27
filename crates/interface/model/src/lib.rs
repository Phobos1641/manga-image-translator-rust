use std::{collections::HashMap, ops::Deref, path::PathBuf, sync::Arc};

use crate::db::ModelDb;
use anyhow::anyhow;
use tokio::sync::{RwLock, RwLockReadGuard};

pub mod db;

pub type ModelWrap<T> = Arc<RwLock<Option<T>>>;
pub struct ModelRead<'a, T> {
    inner: RwLockReadGuard<'a, Option<T>>,
}

impl<'a, T> ModelRead<'a, T> {
    pub fn new(inner: RwLockReadGuard<'a, Option<T>>) -> Self {
        Self { inner }
    }
}

impl<'a, T> Deref for ModelRead<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("RwLock contained None")
    }
}

#[async_trait::async_trait]
pub trait ModelLoad {
    type T;
    async fn loaded(&self) -> bool;
    async fn get_model(&self) -> Option<ModelRead<'_, Self::T>>;
    async fn load(&self) -> anyhow::Result<ModelRead<'_, Self::T>> {
        if self.loaded().await {
            return Ok(self.get_model().await.expect("Checked before"));
        }
        self.reload().await
    }
    async fn reload(&self) -> anyhow::Result<ModelRead<'_, Self::T>>;
}

#[async_trait::async_trait]
pub trait Model {
    fn name(&self) -> &'static str;
    fn kind(&self) -> &'static str;
    fn models(&self) -> HashMap<&'static str, ModelSource>;
    async fn unload(&self);
    async fn download_model(&self, key: &str, file: &str) -> anyhow::Result<PathBuf> {
        let models = self.models();
        let model = models.get(key).ok_or(anyhow!("Model not found"))?;
        ModelDb {}.get(self.kind(), self.name(), file, &model.url, &model.hash)
    }
    async fn loaded_(&self) -> bool;
    async fn reload_(&self) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! impl_model_helpers {
    ($kind:literal, $name:literal, $model:ident) => {
        fn loaded_<'a, 'b>(
            &'a self,
        ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output = bool> + ::core::marker::Send + 'b>,
        >
        where
            'a: 'b,
            Self: 'b,
        {
            Box::pin(async move { self.loaded().await })
        }

        fn reload_<'life0, 'async_trait>(
            &'life0 self,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = anyhow::Result<()>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                self.reload().await?;
                Ok(())
            })
        }

        fn unload<'life0, 'async_trait>(
            &'life0 self,
        ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                *self.$model.write().await = None;
                ()
            })
        }

        fn name(&self) -> &'static str {
            $name
        }

        fn kind(&self) -> &'static str {
            $kind
        }
    };
}

#[macro_export]
macro_rules! impl_model_load_helpers {
    ($model:ident, $kind:ty) => {
        type T = $kind;
        fn loaded<'life0, 'async_trait>(
            &'life0 self,
        ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output = bool> + ::core::marker::Send + 'async_trait>,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move { self.$model.read().await.is_some() })
        }

        fn get_model<'life0, 'async_trait>(
            &'life0 self,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Option<ModelRead<'_, Self::T>>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                let model = self.$model.read().await;
                if model.is_none() {
                    None
                } else {
                    Some(ModelRead::new(model))
                }
            })
        }
    };
}

pub struct ModelSource {
    pub url: &'static str,
    pub hash: &'static str,
}
