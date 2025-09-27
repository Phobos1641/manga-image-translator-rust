use std::{collections::HashMap, fs, path::PathBuf};

use interface_image::{load_mask, load_raw_image, save_mask, save_raw_image, Mask, RawImage};
use log::warn;
use uuid::Uuid;

pub struct ImageStorage {
    path: PathBuf,
    loaded: HashMap<Uuid, Data>,
}

impl ImageStorage {
    pub fn get_image(&mut self, uuid: Uuid) -> &RawImage {
        let not_loaded = matches!(
            self.loaded.get(&uuid).expect("Invlaid uuid"),
            Data::SavedImg
        );
        if not_loaded {
            warn!("image wasnt loaded");
            self.load(uuid.clone());
            self.get_image(uuid)
        } else {
            let img = self.loaded.get(&uuid).expect("Invlaid uuid");

            match img {
                Data::Img(raw_image) => return raw_image,
                Data::SavedImg => unreachable!(),
                Data::SavedMask => panic!("type mismatch"),
                Data::Mask(_) => panic!("type mismatch"),
            }
        }
    }

    pub fn get_mask(&mut self, uuid: Uuid) -> &Mask {
        let not_loaded = matches!(
            self.loaded.get(&uuid).expect("Invlaid uuid"),
            Data::SavedMask
        );
        if not_loaded {
            warn!("mask wasnt loaded");
            self.load(uuid);
            self.get_mask(uuid)
        } else {
            let img = self.loaded.get(&uuid).expect("Invlaid uuid");
            match img {
                Data::Mask(mask) => mask,
                Data::SavedMask => unreachable!(),
                Data::SavedImg => panic!("type mismatch"),
                Data::Img(_) => panic!("type mismatch"),
            }
        }
    }

    pub fn new(path: PathBuf) -> Self {
        fs::create_dir_all(&path).expect("Failed to create storage directory");
        Self {
            path,
            loaded: HashMap::new(),
        }
    }

    pub fn unload(&mut self, uuid: Uuid) {
        if let Some(data) = self.loaded.remove(&uuid) {
            let file_path = self.path.join(format!("{uuid}.tmp.bin"));
            let saved_data = match &data {
                Data::Img(_) => Data::SavedImg,
                Data::Mask(_) => Data::SavedMask,
                _ => unreachable!(),
            };
            match data {
                Data::Img(img) => save_raw_image(&file_path, &img).unwrap(),
                Data::Mask(mask) => save_mask(&file_path, mask).unwrap(),
                Data::SavedImg | Data::SavedMask => return,
            };

            self.loaded.insert(uuid, saved_data);
        }
    }

    pub fn load(&mut self, uuid: Uuid) {
        if let Some(data) = self.loaded.get(&uuid) {
            match data {
                Data::SavedImg | Data::SavedMask => {
                    let file_path = self.path.join(format!("{uuid}.tmp.bin"));
                    let loaded_data = match data {
                        Data::SavedImg => Data::Img(load_raw_image(&file_path).unwrap()),
                        Data::SavedMask => Data::Mask(load_mask(&file_path).unwrap()),
                        _ => unreachable!(),
                    };
                    self.loaded.insert(uuid, loaded_data);
                }
                Data::Img(_) | Data::Mask(_) => {}
            }
        }
    }

    pub fn add_img_cached(&mut self, uuid: Uuid, img: RawImage) -> Uuid {
        let file_path = self.path.join(format!("{uuid}.tmp.bin"));
        save_raw_image(&file_path, &img).unwrap();
        self.loaded.insert(uuid, Data::SavedImg);
        uuid
    }

    pub fn add_mask_cached(&mut self, uuid: Uuid, mask: Mask) -> Uuid {
        let file_path = self.path.join(format!("{uuid}.tmp.bin"));
        save_mask(&file_path, mask);
        self.loaded.insert(uuid, Data::SavedMask);
        uuid
    }

    pub fn add_img(&mut self, img: RawImage) -> Uuid {
        let uuid = Uuid::new_v4();
        self.loaded.insert(uuid, Data::Img(img));
        uuid
    }
    pub fn add_mask(&mut self, mask: Mask) -> Uuid {
        let uuid = Uuid::new_v4();
        self.loaded.insert(uuid, Data::Mask(mask));
        uuid
    }
}

enum Data {
    SavedImg,
    SavedMask,
    Img(RawImage),
    Mask(Mask),
}
