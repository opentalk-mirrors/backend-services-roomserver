// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, marker::PhantomData};

use opentalk_roomserver_signaling::signaling_module::{SignalingModule, SignalingModuleInitData};
use opentalk_types_common::modules::ModuleId;

use super::{ModuleDispatcher, ModuleHandle};

/// A set of initialized modules that can used through their [`ModuleHandle`]
pub type Modules = HashMap<ModuleId, Box<dyn ModuleHandle>>;

pub struct ModuleRegistry {
    modules: HashMap<ModuleId, Box<dyn ModuleInitializer>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    async fn init_module(
        &self,
        module_id: &ModuleId,
        init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>> {
        self.modules.get(module_id)?.init_module(init_data).await
    }

    pub fn add_module<M: SignalingModule + Sync + 'static>(&mut self) {
        self.modules
            .insert(M::NAMESPACE, Box::new(ModuleInitializerImpl::<M>::new()));
    }

    /// Attempt to initialize all given modules
    ///
    /// A given module id might be unknown to the [`ModuleRegistry`] and can't be initialized. A module might also not
    /// be able to initialize with the given [`SignalingModuleInitData`]. Uninitialized modules are returned in a
    /// separate list.
    pub(crate) async fn initialize_modules(
        &self,
        init_data: SignalingModuleInitData,
    ) -> (Modules, Vec<ModuleId>) {
        let mut initializers = Vec::new();

        for module_id in init_data.room_parameters.tariff.modules.keys() {
            let init_data = init_data.clone();

            initializers
                .push(async move { (module_id, self.init_module(module_id, init_data).await) });
        }

        let mut modules = HashMap::new();
        let mut uninitialized = Vec::new();

        for (module_id, module) in futures::future::join_all(initializers).await {
            let module_id = module_id.clone();
            if let Some(module) = module {
                modules.insert(module_id, module);
            } else {
                uninitialized.push(module_id);
            }
        }

        (modules, uninitialized)
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
trait ModuleInitializer: Sync + Send {
    async fn init_module(
        &self,
        init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>>;
}

struct ModuleInitializerImpl<M: SignalingModule + Sync> {
    phantom_data: PhantomData<M>,
}

impl<M: SignalingModule + Sync> ModuleInitializerImpl<M> {
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<M: SignalingModule + Sync + 'static> ModuleInitializer for ModuleInitializerImpl<M> {
    async fn init_module(
        &self,
        init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>> {
        if let Some(module) = M::init(init_data) {
            Some(Box::new(ModuleDispatcher { module }))
        } else {
            tracing::debug!("`{}` module initializer returned none", M::NAMESPACE);
            None
        }
    }
}
