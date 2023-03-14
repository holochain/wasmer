use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::Future;
use tokio::runtime::Handle;
use wasmer::{
    vm::{VMMemory, VMSharedMemory},
    Module, Store,
};

use crate::os::task::thread::WasiThreadError;

use super::{SpawnType, VirtualTaskManager};

/// A task manager that uses tokio to spawn tasks.
#[derive(Clone, Debug)]
pub struct TokioTaskManager(Handle);

static GLOBAL_RUNTIME: Mutex<Option<(Arc<tokio::runtime::Runtime>, Handle)>> = Mutex::new(None);

impl TokioTaskManager {
    pub fn new(rt: Handle) -> Self {
        Self(rt)
    }

    pub fn runtime_handle(&self) -> tokio::runtime::Handle {
        self.0.clone()
    }

    /// Allows the system to set the shared runtime that will be used by other
    /// processes within Wasmer
    pub fn set_shared(rt: Arc<tokio::runtime::Runtime>) -> Result<(), anyhow::Error> {
        let mut guard = GLOBAL_RUNTIME.lock().unwrap();
        if guard.is_some() {
            return Err(anyhow::format_err!("The shared runtime has already been set or lazy initialized - it can not be overridden"));
        }
        guard.replace((rt.clone(), rt.handle().clone()));
        Ok(())
    }

    /// Shared tokio [`Runtime`] that is used by default.
    ///
    /// This exists because a tokio runtime is heavy, and there should not be many
    /// independent ones in a process.
    pub fn shared() -> Self {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            Self(handle)
        } else {
            let mut guard = GLOBAL_RUNTIME.lock().unwrap();
            let rt = guard.get_or_insert_with(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let handle = rt.handle().clone();
                (Arc::new(rt), handle)
            });
            Self(rt.1.clone())
        }
    }
}

impl Default for TokioTaskManager {
    fn default() -> Self {
        Self::shared()
    }
}

struct TokioRuntimeGuard<'g> {
    #[allow(unused)]
    inner: tokio::runtime::EnterGuard<'g>,
}
impl<'g> Drop for TokioRuntimeGuard<'g> {
    fn drop(&mut self) {}
}

#[async_trait::async_trait]
impl VirtualTaskManager for TokioTaskManager {
    fn build_memory(&self, spawn_type: SpawnType) -> Result<Option<VMMemory>, WasiThreadError> {
        match spawn_type {
            SpawnType::CreateWithType(mem) => VMSharedMemory::new(&mem.ty, &mem.style)
                .map_err(|err| {
                    tracing::error!("could not create memory: {err}");
                    WasiThreadError::MemoryCreateFailed
                })
                .map(|m| Some(m.into())),
            SpawnType::NewThread(mem, _) => Ok(Some(mem)),
            SpawnType::Create => Ok(None),
        }
    }

    /// See [`VirtualTaskManager::sleep_now`].
    async fn sleep_now(&self, time: Duration) {
        if time == Duration::ZERO {
            tokio::task::yield_now().await;
        } else {
            tokio::time::sleep(time).await;
        }
    }

    /// See [`VirtualTaskManager::task_shared`].
    fn task_shared(
        &self,
        task: Box<
            dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + 'static,
        >,
    ) -> Result<(), WasiThreadError> {
        self.0.spawn(async move {
            let fut = task();
            fut.await
        });
        Ok(())
    }

    /// See [`VirtualTaskManager::runtime`].
    fn runtime(&self) -> &Handle {
        &self.0
    }

    /// See [`VirtualTaskManager::block_on`].
    #[allow(dyn_drop)]
    fn runtime_enter<'g>(&'g self) -> Box<dyn std::ops::Drop + 'g> {
        Box::new(TokioRuntimeGuard {
            inner: self.0.enter(),
        })
    }

    /// See [`VirtualTaskManager::enter`].
    fn task_wasm(
        &self,
        task: Box<dyn FnOnce(Store, Module, Option<VMMemory>) + Send + 'static>,
        store: Store,
        module: Module,
        spawn_type: SpawnType,
    ) -> Result<(), WasiThreadError> {
        let memory = self.build_memory(spawn_type)?;
        self.0.spawn_blocking(move || {
            // Invoke the callback
            task(store, module, memory);
        });
        Ok(())
    }

    /// See [`VirtualTaskManager::task_dedicated`].
    fn task_dedicated(
        &self,
        task: Box<dyn FnOnce() + Send + 'static>,
    ) -> Result<(), WasiThreadError> {
        self.0.spawn_blocking(move || {
            task();
        });
        Ok(())
    }

    /// See [`VirtualTaskManager::thread_parallelism`].
    fn thread_parallelism(&self) -> Result<usize, WasiThreadError> {
        Ok(std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(8))
    }
}
