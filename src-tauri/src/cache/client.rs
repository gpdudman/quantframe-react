use std::{
    fs::File,
    io::{self, Read, Write},
    path::{self, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    helper, logger, utils::modules::error::AppError, wfm_client::{
        client::WFMClient,
        types::{
            item::Item, riven_attribute_info::RivenAttributeInfo, riven_type_info::RivenTypeInfo,
        },
    }
};

use super::modules::{
    arcane::ArcaneModule, arch_gun::ArchGunModule, arch_melee::ArchMeleeModule,
    archwing::ArchwingModule, fish::FishModule, item::ItemModule, item_price::ItemPriceModule,
    melee::MeleeModule, misc::MiscModule, mods::ModModule, parts::PartModule, pet::PetModule,
    primary::PrimaryModule, resource::ResourceModule, riven::RivenModule,
    secondary::SecondaryModule, sentinel::SentinelModule, skin::SkinModule,
    tradable_items::TradableItemModule, warframe::WarframeModule,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct CacheDataStruct {
    pub last_refresh: Option<String>,
    pub item: CacheDataItemStruct,
    pub riven: CacheDataRivenStruct,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct CacheDataItemStruct {
    pub items: Vec<Item>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheDataRivenStruct {
    pub items: Vec<RivenTypeInfo>,
    pub attributes: Vec<RivenAttributeInfo>,
}

#[derive(Clone, Debug)]
pub struct CacheClient {
    pub log_file: PathBuf,
    pub wfm: Arc<Mutex<WFMClient>>,
    pub qf: Arc<Mutex<crate::qf_client::client::QFClient>>,
    pub cache_data: Arc<Mutex<CacheDataStruct>>,
    item_module: Arc<RwLock<Option<ItemModule>>>,
    item_price_module: Arc<RwLock<Option<ItemPriceModule>>>,
    riven_module: Arc<RwLock<Option<RivenModule>>>,
    arcane_module: Arc<RwLock<Option<ArcaneModule>>>,
    warframe_module: Arc<RwLock<Option<WarframeModule>>>,
    arch_gun_module: Arc<RwLock<Option<ArchGunModule>>>,
    arch_melee_module: Arc<RwLock<Option<ArchMeleeModule>>>,
    archwing_module: Arc<RwLock<Option<ArchwingModule>>>,
    melee_module: Arc<RwLock<Option<MeleeModule>>>,
    mods_module: Arc<RwLock<Option<ModModule>>>,
    primary_module: Arc<RwLock<Option<PrimaryModule>>>,
    secondary_module: Arc<RwLock<Option<SecondaryModule>>>,
    sentinel_module: Arc<RwLock<Option<SentinelModule>>>,
    tradable_items_module: Arc<RwLock<Option<TradableItemModule>>>,
    skin_module: Arc<RwLock<Option<SkinModule>>>,
    misc_module: Arc<RwLock<Option<MiscModule>>>,
    pet_module: Arc<RwLock<Option<PetModule>>>,
    resource_module: Arc<RwLock<Option<ResourceModule>>>,
    part_module: Arc<RwLock<Option<PartModule>>>,
    fish_module: Arc<RwLock<Option<FishModule>>>,
    pub component: String,
    pub cache_path: PathBuf,
    md5_file: String,
}

impl CacheClient {
    pub fn new(
        wfm: Arc<Mutex<WFMClient>>,
        qf: Arc<Mutex<crate::qf_client::client::QFClient>>,
    ) -> Self {
        CacheClient {
            log_file: PathBuf::from("cache"),
            wfm,
            qf,
            cache_data: Arc::new(Mutex::new(CacheDataStruct {
                last_refresh: None,
                item: CacheDataItemStruct { items: vec![] },
                riven: CacheDataRivenStruct {
                    items: vec![],
                    attributes: vec![],
                },
            })),
            component: "Cache".to_string(),
            md5_file: "cache_id.txt".to_string(),
            item_module: Arc::new(RwLock::new(None)),
            item_price_module: Arc::new(RwLock::new(None)),
            riven_module: Arc::new(RwLock::new(None)),
            arcane_module: Arc::new(RwLock::new(None)),
            warframe_module: Arc::new(RwLock::new(None)),
            arch_gun_module: Arc::new(RwLock::new(None)),
            arch_melee_module: Arc::new(RwLock::new(None)),
            archwing_module: Arc::new(RwLock::new(None)),
            melee_module: Arc::new(RwLock::new(None)),
            mods_module: Arc::new(RwLock::new(None)),
            primary_module: Arc::new(RwLock::new(None)),
            secondary_module: Arc::new(RwLock::new(None)),
            sentinel_module: Arc::new(RwLock::new(None)),
            tradable_items_module: Arc::new(RwLock::new(None)),
            skin_module: Arc::new(RwLock::new(None)),
            misc_module: Arc::new(RwLock::new(None)),
            pet_module: Arc::new(RwLock::new(None)),
            resource_module: Arc::new(RwLock::new(None)),
            part_module: Arc::new(RwLock::new(None)),
            fish_module: Arc::new(RwLock::new(None)),
            cache_path: helper::get_app_roaming_path().join("cache"),
        }
    }

    pub fn update_current_cache_id(&self, cache_id: String) -> Result<(), AppError> {
        let cache_path = self.cache_path.join(self.md5_file.clone());
        let mut file = File::create(cache_path)
            .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;

        file.write_all(cache_id.as_bytes())
            .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;

        Ok(())
    }

    fn get_current_cache_id(&self) -> Result<String, AppError> {
        let cache_path = self.cache_path.join(self.md5_file.clone());
        if !cache_path.exists() {
            return Ok("N/A".to_string());
        }
        let mut file = File::open(cache_path)
            .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;
        Ok(content)
    }

    pub async fn download_cache_data(&self) -> Result<(), AppError> {
        let qf = self.qf.lock()?.clone();
        let zip_data = qf.cache().get_zip().await?;

        let reader = std::io::Cursor::new(zip_data);
        let mut archive = zip::ZipArchive::new(reader)
            .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;

        let extract_to = helper::get_app_roaming_path();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;
            let output_path = extract_to.join(file.mangled_name());

            if file.is_dir() {
                std::fs::create_dir_all(&output_path)
                    .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;
            } else {
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;
                    }
                }

                let mut output_file = File::create(&output_path)
                    .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;

                std::io::copy(&mut file, &mut output_file)
                    .map_err(|e| AppError::new(&self.component, eyre!(e.to_string())))?;
            }
        }
        logger::info_con(&self.component, "Cache data downloaded and extracted");
        Ok(())
    }

    fn get_file_path() -> PathBuf {
        let app_path = helper::get_app_roaming_path();
        let settings_path = app_path.join("cache.json");
        settings_path
    }

    pub async fn load(&self) -> Result<CacheDataStruct, AppError> {
        let qf = self.qf.lock()?.clone();

        let current_cache_id = self.get_current_cache_id()?;
        logger::info_con(
            &self.component,
            format!("Current cache id: {}", current_cache_id).as_str(),
        );
        let remote_cache_id = match qf.cache().get_cache_id().await {
            Ok(id) => id,
            Err(e) => {
                logger::error_con(
                    &self.component,
                    format!(
                        "There was an error downloading the cache from the server: {:?}",
                        e
                    )
                    .as_str(),
                );
                logger::info_con(&self.component, "Using the current cache data");
                current_cache_id.clone()
            }
        };
        logger::info_con(
            &self.component,
            format!("Remote cache id: {}", remote_cache_id).as_str(),
        );
        if current_cache_id != remote_cache_id {
            logger::info_con(
                &self.component,
                "Cache id mismatch, downloading new cache data",
            );
            self.download_cache_data().await?;
            self.update_current_cache_id(remote_cache_id)?;
        }

        self.arcane().load()?;
        self.warframe().load()?;
        self.arch_gun().load()?;
        self.arch_melee().load()?;
        self.archwing().load()?;
        self.melee().load()?;
        self.mods().load()?;
        self.primary().load()?;
        self.secondary().load()?;
        self.sentinel().load()?;
        self.tradable_items().load()?;
        self.skin().load()?;
        self.misc().load()?;
        self.pet().load()?;
        self.fish().load()?;
        self.resource().load()?;
        self.riven().load()?;
        self.parts().load()?;
        self.item_price().load().await?;

        let path_ref = Self::get_file_path();

        if path_ref.exists() {
            let (se, vaild) = Self::read_from_file()?;
            if vaild {
                let last_refresh = se.last_refresh.clone();
                match last_refresh {
                    Some(last_refresh) => {
                        let last_refresh = chrono::DateTime::parse_from_rfc3339(&last_refresh)
                            .map_err(|e| AppError::new("Cache", eyre!(e.to_string())))?;
                        let now = chrono::Utc::now();
                        let diff = now.signed_duration_since(last_refresh);
                        if diff.num_hours() < 24 {
                            let arced_mutex = Arc::clone(&self.cache_data);
                            let mut my_lock = arced_mutex.lock()?;
                            my_lock.last_refresh = Some(last_refresh.to_string());
                            my_lock.item = se.item;
                            my_lock.riven = se.riven;
                            return Ok(my_lock.clone());
                        } else {
                            let data = self.refresh().await?;
                            self.save_to_file()?;
                            return Ok(data);
                        }
                    }
                    None => {
                        let data = self.refresh().await?;
                        self.save_to_file()?;
                        return Ok(data);
                    }
                }
            } else {
                let data = self.refresh().await?;
                self.save_to_file()?;
                return Ok(data);
            }
        } else {
            let data = self.refresh().await?;
            self.save_to_file()?;
            return Ok(data);
        }
    }

    pub async fn refresh(&self) -> Result<CacheDataStruct, AppError> {
        self.item().refresh().await?;
        self.riven().refresh().await?;
        self.set_last_refresh(chrono::Utc::now().to_rfc3339())?;
        let cache_data = self.cache_data.lock()?.clone();
        Ok(cache_data)
    }

    pub fn item(&self) -> ItemModule {
        // Lazily initialize ItemModule if not already initialized
        if self.item_module.read().unwrap().is_none() {
            *self.item_module.write().unwrap() = Some(ItemModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.item_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_item_module(&self, module: ItemModule) {
        // Update the stored ItemModule
        *self.item_module.write().unwrap() = Some(module);
    }

    pub fn item_price(&self) -> ItemPriceModule {
        // Lazily initialize ItemModule if not already initialized
        if self.item_price_module.read().unwrap().is_none() {
            *self.item_price_module.write().unwrap() =
                Some(ItemPriceModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.item_price_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_item_price_module(&self, module: ItemPriceModule) {
        // Update the stored ItemModule
        *self.item_price_module.write().unwrap() = Some(module);
    }

    pub fn riven(&self) -> RivenModule {
        // Lazily initialize ItemModule if not already initialized
        if self.riven_module.read().unwrap().is_none() {
            *self.riven_module.write().unwrap() = Some(RivenModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.riven_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_riven_module(&self, module: RivenModule) {
        // Update the stored ItemModule
        *self.riven_module.write().unwrap() = Some(module);
    }

    pub fn arcane(&self) -> ArcaneModule {
        // Lazily initialize ArcaneModule if not already initialized
        if self.arcane_module.read().unwrap().is_none() {
            *self.arcane_module.write().unwrap() = Some(ArcaneModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the arcane_module is initialized
        self.arcane_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_arcane_module(&self, module: ArcaneModule) {
        // Update the stored ArcaneModule
        *self.arcane_module.write().unwrap() = Some(module);
    }

    pub fn arch_gun(&self) -> ArchGunModule {
        // Lazily initialize ArchGunModule if not already initialized
        if self.arch_gun_module.read().unwrap().is_none() {
            *self.arch_gun_module.write().unwrap() = Some(ArchGunModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the arch_gun_module is initialized
        self.arch_gun_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_arch_gun_module(&self, module: ArchGunModule) {
        // Update the stored ArchGunModule
        *self.arch_gun_module.write().unwrap() = Some(module);
    }

    pub fn arch_melee(&self) -> ArchMeleeModule {
        // Lazily initialize ArchMeleeModule if not already initialized
        if self.arch_melee_module.read().unwrap().is_none() {
            *self.arch_melee_module.write().unwrap() =
                Some(ArchMeleeModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the arch_melee_module is initialized
        self.arch_melee_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_arch_melee_module(&self, module: ArchMeleeModule) {
        // Update the stored ArchMeleeModule
        *self.arch_melee_module.write().unwrap() = Some(module);
    }

    pub fn archwing(&self) -> ArchwingModule {
        // Lazily initialize ArchwingModule if not already initialized
        if self.archwing_module.read().unwrap().is_none() {
            *self.archwing_module.write().unwrap() =
                Some(ArchwingModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the archwing_module is initialized
        self.archwing_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_archwing_module(&self, module: ArchwingModule) {
        // Update the stored ArchwingModule
        *self.archwing_module.write().unwrap() = Some(module);
    }

    pub fn melee(&self) -> MeleeModule {
        // Lazily initialize MeleeModule if not already initialized
        if self.melee_module.read().unwrap().is_none() {
            *self.melee_module.write().unwrap() = Some(MeleeModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the melee_module is initialized
        self.melee_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_melee_module(&self, module: MeleeModule) {
        // Update the stored MeleeModule
        *self.melee_module.write().unwrap() = Some(module);
    }

    pub fn mods(&self) -> ModModule {
        // Lazily initialize ModModule if not already initialized
        if self.mods_module.read().unwrap().is_none() {
            *self.mods_module.write().unwrap() = Some(ModModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the mods_module is initialized
        self.mods_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_mods_module(&self, module: ModModule) {
        // Update the stored ModModule
        *self.mods_module.write().unwrap() = Some(module);
    }

    pub fn primary(&self) -> PrimaryModule {
        // Lazily initialize PrimaryModule if not already initialized
        if self.primary_module.read().unwrap().is_none() {
            *self.primary_module.write().unwrap() = Some(PrimaryModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the primary_module is initialized
        self.primary_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_primary_module(&self, module: PrimaryModule) {
        // Update the stored PrimaryModule
        *self.primary_module.write().unwrap() = Some(module);
    }

    pub fn secondary(&self) -> SecondaryModule {
        // Lazily initialize SecondaryModule if not already initialized
        if self.secondary_module.read().unwrap().is_none() {
            *self.secondary_module.write().unwrap() =
                Some(SecondaryModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the secondary_module is initialized
        self.secondary_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_secondary_module(&self, module: SecondaryModule) {
        // Update the stored SecondaryModule
        *self.secondary_module.write().unwrap() = Some(module);
    }

    pub fn sentinel(&self) -> SentinelModule {
        // Lazily initialize SentinelModule if not already initialized
        if self.sentinel_module.read().unwrap().is_none() {
            *self.sentinel_module.write().unwrap() =
                Some(SentinelModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the sentinel_module is initialized
        self.sentinel_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_sentinel_module(&self, module: SentinelModule) {
        // Update the stored SentinelModule
        *self.sentinel_module.write().unwrap() = Some(module);
    }

    pub fn warframe(&self) -> WarframeModule {
        // Lazily initialize ArcaneModule if not already initialized
        if self.warframe_module.read().unwrap().is_none() {
            *self.warframe_module.write().unwrap() =
                Some(WarframeModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the warframe_module is initialized
        self.warframe_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_warframe_module(&self, module: WarframeModule) {
        // Update the stored WarframeModule
        *self.warframe_module.write().unwrap() = Some(module);
    }

    pub fn tradable_items(&self) -> TradableItemModule {
        // Lazily initialize ArcaneModule if not already initialized
        if self.tradable_items_module.read().unwrap().is_none() {
            *self.tradable_items_module.write().unwrap() =
                Some(TradableItemModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the tradable_items_module is initialized
        self.tradable_items_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_tradable_items_module(&self, module: TradableItemModule) {
        // Update the stored Warframe
        *self.tradable_items_module.write().unwrap() = Some(module);
    }

    pub fn resource(&self) -> ResourceModule {
        // Lazily initialize ResourceModule if not already initialized
        if self.resource_module.read().unwrap().is_none() {
            *self.resource_module.write().unwrap() =
                Some(ResourceModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.resource_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_resource_module(&self, module: ResourceModule) {
        // Update the stored ResourceModule
        *self.resource_module.write().unwrap() = Some(module);
    }

    pub fn misc(&self) -> MiscModule {
        // Lazily initialize MiscModule if not already initialized
        if self.misc_module.read().unwrap().is_none() {
            *self.misc_module.write().unwrap() = Some(MiscModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.misc_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_misc_module(&self, module: MiscModule) {
        // Update the stored MiscModule
        *self.misc_module.write().unwrap() = Some(module);
    }

    pub fn pet(&self) -> PetModule {
        // Lazily initialize PetModule if not already initialized
        if self.pet_module.read().unwrap().is_none() {
            *self.pet_module.write().unwrap() = Some(PetModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.pet_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_pet_module(&self, module: PetModule) {
        // Update the stored PetModule
        *self.pet_module.write().unwrap() = Some(module);
    }

    pub fn fish(&self) -> FishModule {
        // Lazily initialize FishModule if not already initialized
        if self.fish_module.read().unwrap().is_none() {
            *self.fish_module.write().unwrap() = Some(FishModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.fish_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_fish_module(&self, module: FishModule) {
        // Update the stored FishModule
        *self.fish_module.write().unwrap() = Some(module);
    }

    pub fn skin(&self) -> SkinModule {
        // Lazily initialize SkinModule if not already initialized
        if self.skin_module.read().unwrap().is_none() {
            *self.skin_module.write().unwrap() = Some(SkinModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.skin_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_skin_module(&self, module: SkinModule) {
        // Update the stored SkinModule
        *self.skin_module.write().unwrap() = Some(module);
    }

    pub fn parts(&self) -> PartModule {
        // Lazily initialize PartModule if not already initialized
        if self.part_module.read().unwrap().is_none() {
            *self.part_module.write().unwrap() = Some(PartModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the order_module is initialized
        self.part_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_part_module(&self, module: PartModule) {
        // Update the stored PartModule
        *self.part_module.write().unwrap() = Some(module);
    }

    pub fn set_last_refresh(&self, last_refresh: String) -> Result<(), AppError> {
        let arced_mutex = Arc::clone(&self.cache_data);
        let mut my_lock = arced_mutex.lock()?;
        my_lock.last_refresh = Some(last_refresh);
        Ok(())
    }

    pub fn save_to_file(&self) -> Result<(), AppError> {
        let chache_data = self.cache_data.clone();
        let json = serde_json::to_string_pretty(&chache_data)
            .map_err(|e| AppError::new("Cache", eyre!(e.to_string())))?;

        let mut file = File::create(Self::get_file_path())
            .map_err(|e| AppError::new("Cache", eyre!(e.to_string())))?;

        file.write_all(json.as_bytes())
            .map_err(|e| AppError::new("Cache", eyre!(e.to_string())))?;

        Ok(())
    }

    pub fn get_path(&self, path: &str) -> PathBuf {
        let path = self.cache_path.join(path);
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Failed to create cache directory");
        }
        path
    }

    pub fn read_text_from_file(&self, path: &PathBuf) -> Result<String, AppError> {
        let mut file = File::open(self.cache_path.join(path))
            .map_err(|e| AppError::new(&self.component, eyre!(format!("Failed to open file: {}, error: {}", path.to_str().unwrap(), e.to_string()))))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| AppError::new(&self.component, eyre!(format!("Failed to read file: {}, error: {}", path.to_str().unwrap(), e.to_string()))) )?;

        Ok(content)
    }

    pub fn read_from_file() -> Result<(CacheDataStruct, bool), AppError> {
        let mut file = File::open(Self::get_file_path())
            .map_err(|e| AppError::new("Cache", eyre!(format!("Failed to open file: {}, error: {}", Self::get_file_path().to_str().unwrap(), e.to_string()))))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| AppError::new("Cache", eyre!(format!("Failed to read file: {}, error: {}", Self::get_file_path().to_str().unwrap(), e.to_string()))) )?;

        Ok(Self::validate_json(&content)?)
    }

    fn validate_json(json_str: &str) -> Result<(CacheDataStruct, bool), AppError> {
        let mut is_valid = true;
        // Parse the JSON string into a Value object
        let mut json_value: Value = serde_json::from_str(json_str)
            .map_err(|e| AppError::new("Cache", eyre!(e.to_string())))?;

        if json_value.get("last_refresh").is_none() {
            let now = chrono::Utc::now();
            // Set the 'last_refresh' property to None
            json_value["last_refresh"] = json!(now.to_rfc3339());
            is_valid = false;
        }

        // Check for nested properties within 'item'
        if let Some(item_data) = json_value.get_mut("item") {
            if item_data.get("items").is_none() {
                item_data["items"] = json!([]);
                is_valid = false;
            }
        }

        // Check for nested properties within 'riven'
        if let Some(riven_data) = json_value.get_mut("riven") {
            if riven_data.get("items").is_none() {
                riven_data["items"] = json!([]);
                is_valid = false;
            }
            if riven_data.get("attributes").is_none() {
                riven_data["attributes"] = json!([]);
                is_valid = false;
            }
        }

        // Deserialize the updated JSON object into a SettingsState struct
        let deserialized: CacheDataStruct = serde_json::from_value(json_value)
            .map_err(|e| AppError::new("Settings", eyre!(e.to_string())))?;
        Ok((deserialized, is_valid))
    }
}
