use crate::file_readers::wad::Wad;
use crate::utils;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

pub type WadCacheMap = HashMap<String, HashMap<String, i64>>;

pub struct CacheHandler {
    wad_cache: Option<WadCacheMap>,
    template_ids: Option<HashMap<String, String>>,
    _root_wad: Wad,
}

impl CacheHandler {
    pub fn new() -> io::Result<Self> {
        let root_wad = Wad::from_game_data("Root")?;
        Ok(Self {
            wad_cache: None,
            template_ids: None,
            _root_wad: root_wad,
        })
    }

    pub fn install_location(&self) -> io::Result<PathBuf> {
        utils::get_wiz_install()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }

    pub fn cache_dir(&self) -> io::Result<PathBuf> {
        utils::get_cache_folder()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }

    pub fn get_wad_cache(&mut self) -> io::Result<WadCacheMap> {
        let cache_file = self.cache_dir()?.join("wad_cache.data");

        if let Ok(data) = fs::read_to_string(&cache_file) {
            if let Ok(cache) = serde_json::from_str(&data) {
                return Ok(cache);
            }
        }

        Ok(HashMap::new())
    }

    pub fn write_wad_cache(&mut self) -> io::Result<()> {
        let cache_file = self.cache_dir()?.join("wad_cache.data");
        if let Some(ref cache) = self.wad_cache {
            let data = serde_json::to_string(cache)?;
            fs::write(cache_file, data)?;
        }
        Ok(())
    }

    fn check_updated_inner(
        &mut self,
        wad_file: &mut Wad,
        files: &[String],
    ) -> io::Result<Vec<String>> {
        if self.wad_cache.is_none() {
            self.wad_cache = Some(self.get_wad_cache()?);
        }

        let mut res = Vec::new();

        for file_name in files {
            if let Ok(file_info) = wad_file.get_file_info(file_name) {
                let wad_name = wad_file.name.clone();
                let wad_cache = self.wad_cache.as_mut().unwrap();

                let wad_entry = wad_cache.entry(wad_name).or_insert_with(HashMap::new);
                let current_size = *wad_entry.get(file_name).unwrap_or(&-1);

                if current_size != file_info.size as i64 {
                    // tracing::info!("{} has updated. old: {} new: {}", file_name, current_size, file_info.size);
                    res.push(file_name.clone());
                    wad_entry.insert(file_name.clone(), file_info.size as i64);
                } else {
                    // tracing::info!("{} has not updated from {}", file_name, file_info.size);
                }
            }
        }

        Ok(res)
    }

    pub fn check_updated(
        &mut self,
        wad_file: &mut Wad,
        files: &[String],
    ) -> io::Result<Vec<String>> {
        let res = self.check_updated_inner(wad_file, files)?;

        if !res.is_empty() {
            self.write_wad_cache()?;
        }

        Ok(res)
    }

    pub fn cache(&mut self) -> io::Result<()> {
        // tracing::info!("Caching template if needed");
        let mut root_wad = Wad::from_game_data("Root")?;
        self.cache_template(&mut root_wad)?;
        Ok(())
    }

    fn cache_template(&mut self, root_wad: &mut Wad) -> io::Result<()> {
        let updated = self.check_updated(root_wad, &["TemplateManifest.xml".to_string()])?;

        if !updated.is_empty() {
            let file_data = root_wad.get_file("TemplateManifest.xml")?;
            let parsed_template_ids = utils::parse_template_id_file(&file_data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

            let cache_file = self.cache_dir()?.join("template_ids.json");
            let json_data = serde_json::to_string(&parsed_template_ids)?;
            fs::write(cache_file, json_data)?;
        }

        Ok(())
    }

    pub fn get_template_ids(&mut self) -> io::Result<HashMap<String, String>> {
        if self.template_ids.is_none() {
            self.cache()?;
            let cache_file = self.cache_dir()?.join("template_ids.json");
            let data = fs::read_to_string(cache_file)?;
            let ids: HashMap<String, String> = serde_json::from_str(&data)?;
            self.template_ids = Some(ids);
        }

        Ok(self.template_ids.clone().unwrap())
    }

    fn parse_lang_file(file_data: &[u8]) -> Option<HashMap<String, HashMap<String, String>>> {
        // Handle utf-16
        // NOTE: we just treat as utf-16le here. Could check BOM.
        let u16_data: Vec<u16> = file_data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        let decoded = String::from_utf16(&u16_data).ok()?;

        let mut lines = decoded.split("\r\n");
        let header = lines.next()?;
        let parts: Vec<&str> = header.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let lang_name = parts[1].trim().to_string();
        let mut lang_mapping = HashMap::new();

        let remaining_lines: Vec<&str> = lines.collect();
        for chunk in remaining_lines.chunks(3) {
            if chunk.len() >= 3 {
                lang_mapping.insert(chunk[0].trim().to_string(), chunk[2].trim().to_string());
            }
        }

        let mut map = HashMap::new();
        map.insert(lang_name, lang_mapping);
        Some(map)
    }

    fn get_all_lang_file_names(&mut self, root_wad: &mut Wad) -> io::Result<Vec<String>> {
        let names = root_wad.names()?;
        Ok(names
            .into_iter()
            .filter(|n| n.starts_with("Locale/en-US/"))
            .collect())
    }

    fn read_lang_file(
        &mut self,
        root_wad: &mut Wad,
        lang_file: &str,
    ) -> io::Result<Option<HashMap<String, HashMap<String, String>>>> {
        let data = root_wad.get_file(lang_file)?;
        Ok(Self::parse_lang_file(&data))
    }

    fn cache_lang_file(&mut self, root_wad: &mut Wad, lang_file: &str) -> io::Result<()> {
        let updated = self.check_updated(root_wad, &[lang_file.to_string()])?;
        if updated.is_empty() {
            return Ok(());
        }

        if let Some(parsed_lang) = self.read_lang_file(root_wad, lang_file)? {
            let mut lang_map = self.get_langcode_map()?;
            for (k, v) in parsed_lang {
                lang_map.insert(k, v);
            }

            let cache_file = self.cache_dir()?.join("langmap.json");
            let json_data = serde_json::to_string(&lang_map)?;
            fs::write(cache_file, json_data)?;
        }

        Ok(())
    }

    fn cache_lang_files(&mut self, root_wad: &mut Wad) -> io::Result<()> {
        let lang_file_names = self.get_all_lang_file_names(root_wad)?;
        let mut parsed_lang_map = HashMap::new();

        for file_name in lang_file_names {
            let updated = self.check_updated_inner(root_wad, &[file_name.clone()])?;
            if updated.is_empty() {
                continue;
            }

            if let Some(parsed) = self.read_lang_file(root_wad, &file_name)? {
                for (k, v) in parsed {
                    parsed_lang_map.insert(k, v);
                }
            }
        }

        self.write_wad_cache()?;

        let mut lang_map = self.get_langcode_map()?;
        for (k, v) in parsed_lang_map {
            lang_map.insert(k, v);
        }

        let cache_file = self.cache_dir()?.join("langmap.json");
        let json_data = serde_json::to_string(&lang_map)?;
        fs::write(cache_file, json_data)?;

        Ok(())
    }

    pub fn cache_all_langcode_maps(&mut self) -> io::Result<()> {
        let mut root_wad = Wad::from_game_data("Root")?;
        self.cache_lang_files(&mut root_wad)
    }

    pub fn get_langcode_map(&self) -> io::Result<HashMap<String, HashMap<String, String>>> {
        let cache_file = self.cache_dir()?.join("langmap.json");
        if let Ok(data) = fs::read_to_string(cache_file) {
            if let Ok(map) = serde_json::from_str(&data) {
                return Ok(map);
            }
        }
        Ok(HashMap::new())
    }

    pub fn get_template_name(&mut self, template_id: u32) -> io::Result<Option<String>> {
        let template_ids = self.get_template_ids()?;
        Ok(template_ids.get(&template_id.to_string()).cloned())
    }

    pub fn get_langcode_name(&mut self, langcode: &str) -> io::Result<String> {
        let split_point = langcode.find('_').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid langcode format")
        })?;

        let lang_filename = &langcode[..split_point];
        let code = &langcode[split_point + 1..];

        let mut root_wad = Wad::from_game_data("Root")?;
        let lang_files = self.get_all_lang_file_names(&mut root_wad)?;

        let mut cached = false;
        let expected_filename = format!("Locale/en-US/{}.lang", lang_filename);
        for filename in lang_files {
            if filename == expected_filename {
                self.cache_lang_file(&mut root_wad, &filename)?;
                cached = true;
                break;
            }
        }

        if !cached {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No lang file named {}", lang_filename),
            ));
        }

        let langcode_map = self.get_langcode_map()?;
        let lang_file = langcode_map.get(lang_filename).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("No lang file named {}", lang_filename),
            )
        })?;

        let lang_name = lang_file.get(code).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("No lang name with code {}", code),
            )
        })?;

        Ok(lang_name.clone())
    }
}
