use askama::Template;
use super::Result;
use std::{
    fs::File,
    io::Write,
    path::Path,
};

fn write_only_if_changed(path: &Path, doit: impl FnOnce() -> Result<String>) -> Result<()> {
    //let prev = std::fs::read_to_string(path).ok();
    let next = doit()?;
    //if prev.as_ref() != Some(&next) {
        std::fs::write(path, &next)?;
    //}
    Ok(())
}

pub struct CppHeader {
    pub is_pub: bool,
    pub contents: String,
}

pub struct CppSource {
    pub contents: String,
}

pub enum CppItem {
    Header(CppHeader),
    Source(CppSource),
}

pub enum ModuleCppSources<'a> {
    None,
    WithDefaultModule(Vec<(&'a str, Vec<CppItem>)>),
    WithoutDefaultModule(Vec<(&'a str, Vec<CppItem>)>)
}

pub struct AndroidConfig<'a> {
    pub permissions: &'a [&'a str],
}

pub struct ModuleCppImpl<'a> {
    pub pub_includes: &'a[&'a str],
    pub priv_includes: &'a[&'a str],
}

#[derive(Debug)]
pub enum HostType {
    Runtime,  
    RuntimeNoCommandlet,  
    RuntimeAndProgram,  
    CookedOnly,  
    UncookedOnly,  
    Developer,  
    DeveloperTool,  
    Editor,  
    EditorNoCommandlet,  
    EditorAndProgram,  
    Program,  
    ServerOnly,  
    ClientOnly,  
    ClientOnlyNoCommandlet, 
}
impl std::fmt::Display for HostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug)]
pub enum LoadingPhase {
    EarliestPossible,  
    PostConfigInit,  
    PostSplashScreen,  
    PreEarlyLoadingScreen,  
    PreLoadingScreen,  
    PreDefault,  
    Default,  
    PostDefault,  
    PostEngineInit,  
    None,
}
impl std::fmt::Display for LoadingPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

pub struct Module<'a> {
    pub name: &'a str,
    pub android: Option<AndroidConfig<'a>>,
    pub pub_dep_mods: &'a [&'a str],
    pub priv_dep_mods: &'a [&'a str],
    pub pub_include_paths: &'a [&'a str],
    pub priv_include_paths: &'a [&'a str],
    pub priv_defs: &'a [(&'a str, &'a str)],
    pub whitelist_platforms: &'a [&'a str],
    pub external_dylibs: &'a[&'a str],
    pub ty: HostType,
    pub loading_phase: LoadingPhase,
    pub sources: ModuleCppSources<'a>,
}

struct PluginDep {
    name: String,
    enabled: bool,
    whitelist_platforms: Vec<String>,
    blacklist_targets: Vec<String>,
}

pub struct Builder<'a> {
    name: &'a str,
    created_by: &'a str,
    created_by_url: &'a str,
    version: u32,
    version_name: &'a str,
    category: &'a str,
    description: &'a str,
    modules: Vec<Module<'a>>,
    plugin_deps: Vec<PluginDep>,
    out_dir: Option<&'a Path>,
    rs_out_dir: Option<&'a str>,
    icon: Option<&'a[u8]>,
    enabled: bool,
    docs_url: &'a str,
    marketplace_url: &'a str,
    support_url: &'a str,
    can_contain_content: bool,
    is_beta_version: bool,
    enabled_by_default: bool,
    installed: bool,
}

impl<'a> Builder<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            created_by: "",
            created_by_url: "",
            version: 1,
            version_name: "",
            category: "",
            description: "",
            modules: vec![],
            plugin_deps: vec![],
            out_dir: None,
            rs_out_dir: None,
            icon: None,
            enabled: true,
            docs_url: "",
            marketplace_url: "",
            support_url: "",
            can_contain_content: false,
            is_beta_version: false,
            installed: false,
            enabled_by_default: false,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    fn _write_module_h(
        module_name: &str,
        contents: Option<&str>,
        mut out: impl Write,
    ) -> Result<()> {
        if let Some(c) = contents {
            out.write(c.as_bytes()).unwrap();
        } else {
            #[derive(Template)]
            #[template(path = "DefaultModule.h.jinja", escape = "none")]
            struct HeaderTemplate<'a> {
                module_name: &'a str,
            }
            out.write(HeaderTemplate { module_name }.render()?.as_bytes()).unwrap();
        }
        Ok(())
    }

    fn _write_module_cpp(
        module_name: &str,
        contents: Option<&str>,
        mut out: impl Write,
    ) -> Result<()> {
        if let Some(c) = contents {
            out.write(c.as_bytes()).unwrap();
        } else {
            #[derive(Template)]
            #[template(path = "DefaultModule.cpp.jinja", escape = "none")]
            struct SourceTemplate<'a> {
                module_name: &'a str,
            }
            out.write(SourceTemplate { module_name }.render()?.as_bytes()).unwrap();
        }
        Ok(())
    }

    fn write_base_apl(
        android_permission_names: &[&str],
        dylibs: &[&str],
        mut out: impl Write,
    ) -> Result<()> {
        #[derive(Template)]
        #[template(path = "BaseAPL.xml.jinja", escape = "none")]
        struct XmlTemplate<'a> {
            permission_names: &'a[&'a str],
            dylibs: &'a[&'a str]
        }
        out.write(XmlTemplate {
            permission_names: android_permission_names,
            dylibs
        }.render().unwrap().as_bytes()).unwrap();
        Ok(())
    }

    fn write_build(
        dylibs: &[&str],
        module_name: &str,
        pub_dep_mods: &[&str],
        priv_dep_mods: &[&str],
        pub_include_paths: &[&str],
        priv_include_paths: &[&str],
        priv_defs: &[(&str, &str)],
    ) -> Result<String> {
        let pub_deps = pub_dep_mods
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(",");
        let priv_deps = priv_dep_mods
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(",");
        let pub_inc = pub_include_paths
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(",");
        let priv_inc = priv_include_paths
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(",");
        let priv_defs = priv_defs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>();

        #[derive(Template)]
        #[template(path = "Default.build.cs.jinja", escape = "none")]
        struct BuildTemplate<'a> {
            module_name: &'a str,
            pub_deps: &'a str,
            priv_deps: &'a str,
            pub_inc: &'a str,
            priv_inc: &'a str,
            priv_defs: &'a[String],
            dylibs: &'a[&'a str]
        }

        Ok(BuildTemplate {
            module_name,
            pub_deps: &pub_deps,
            priv_deps: &priv_deps,
            pub_inc: &pub_inc,
            priv_inc: &priv_inc,
            priv_defs: &priv_defs,
            dylibs
        }.render().unwrap())
    }

    fn write_plugin(&self, modules: &[&Module], plugin_infos: &[PluginDep]) -> Result<String> {
        let plugins = plugin_infos.into_iter().map(|i| {
            let mut props = vec![];
            props.push(("Name", format!("\"{}\"", i.name)));
            props.push((
                "Enabled",
                if i.enabled { "true" } else { "false" }.to_string(),
            ));
            if !i.whitelist_platforms.is_empty() {
                props.push((
                    "WhitelistPlatforms",
                    i.whitelist_platforms
                        .iter()
                        .map(|p| format!("\"{}\"", p))
                        .collect::<Vec<_>>()
                        .join(","),
                ));
            }
            if !i.blacklist_targets.is_empty() {
                props.push((
                    "BlacklistTargets",
                    i.blacklist_targets
                        .iter()
                        .map(|p| format!("\"{}\"", p))
                        .collect::<Vec<_>>()
                        .join(","),
                ));
            }
            format!(
                "{{\n{}\n    }}",
                props
                    .into_iter()
                    .map(|(k, v)| format!("        \"{}\": {}", k, v))
                    .collect::<Vec<_>>()
                    .join(",\n")
            )
        });

        #[derive(Template)]
        #[template(path = "Default.uplugin.jinja", escape = "none")]
        struct UPluginTemplate<'a> {
            file_version: u32,
            version: u32,
            version_name: &'a str,
            friendly_name: &'a str,
            description: &'a str,
            category: &'a str,
            created_by: &'a str,
            created_by_url: &'a str,
            docs_url: &'a str,
            marketplace_url: &'a str,
            support_url: &'a str,
            can_contain_content: bool,
            is_beta_version: bool,
            installed: bool,
            enabled_by_default: bool,
            plugins: &'a str,
            modules: &'a[&'a Module<'a>],
        }
        Ok(UPluginTemplate {
            file_version: 3,
            version: self.version,
            version_name: self.version_name,
            friendly_name: self.name,
            description: self.description,
            category: self.category,
            created_by: self.created_by,
            created_by_url: self.created_by_url,
            docs_url: self.docs_url,
            marketplace_url: self.marketplace_url,
            support_url: self.support_url,
            can_contain_content: self.can_contain_content,
            is_beta_version: self.is_beta_version,
            installed: self.installed,
            enabled_by_default: self.enabled_by_default,
            plugins: &plugins.collect::<Vec<_>>().join(", "),
            modules,
        }.render().unwrap())
    }

    fn write_icon(icon_bytes: Option<&[u8]>, mut out: impl Write) -> Result<()> {
        // data for an empty 214x183 png file
        let bytes = icon_bytes.unwrap_or_else(|| include_bytes!("../Icon128.png"));
        out.write_all(bytes).unwrap();
        Ok(())
    }

    pub fn created_by(mut self, v: impl Into<&'a str>) -> Self {
        self.created_by = v.into();
        self
    }
    pub fn created_by_url(mut self, v: impl Into<&'a str>) -> Self {
        self.created_by_url = v.into();
        self
    }
    pub fn category(mut self, v: impl Into<&'a str>) -> Self {
        self.category = v.into();
        self
    }
    pub fn version(mut self, v: u32) -> Self {
        self.version = v;
        self
    }
    pub fn version_name(mut self, v: impl Into<&'a str>) -> Self {
        self.version_name = v.into();
        self
    }
    pub fn description(mut self, v: impl Into<&'a str>) -> Self {
        self.description = v.into();
        self
    }
    pub fn module(mut self, module: Module<'a>) -> Self {
        self.modules.push(module);
        self
    }
    pub fn out_dir(mut self, out_dir: &'a Path) -> Self {
        self.out_dir = Some(out_dir);
        self
    }
    pub fn rs_out_dir(mut self, rs_out_dir: impl Into<&'a str>) -> Self {
        self.rs_out_dir = Some(rs_out_dir.into());
        self
    }
    pub fn icon(mut self, bytes: &'a [u8]) -> Self {
        self.icon = Some(bytes);
        self
    }
    pub fn docs_url(mut self, url: &'a str) -> Self {
        self.docs_url = url;
        self
    }
    pub fn marketplace_url(mut self, url: &'a str) -> Self {
        self.marketplace_url = url;
        self
    }
    pub fn support_url(mut self, url: &'a str) -> Self {
        self.support_url = url;
        self
    }
    pub fn can_contain_content(mut self, value: bool) -> Self {
        self.can_contain_content = value;
        self
    }
    pub fn is_beta_version(mut self, value: bool) -> Self {
        self.is_beta_version = value;
        self
    }
    pub fn enabled_by_default(mut self, value: bool) -> Self {
        self.enabled_by_default = value;
        self
    }
    pub fn installed(mut self, value: bool) -> Self {
        self.installed = value;
        self
    }
    pub fn add_plugin(
        mut self,
        name: &str,
        enabled: bool,
        whitelist_platforms: &[&str],
        blacklist_targets: &[&str],
    ) -> Self {
        self.plugin_deps.push(PluginDep {
            name: name.to_string(),
            enabled,
            whitelist_platforms: whitelist_platforms
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>(),
            blacklist_targets: blacklist_targets
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>(),
        });
        self
    }
    pub fn generate(self) -> Result<()> {
        if !self.enabled { return Ok(()) }

        let outdir = match self.out_dir.as_ref() {
            Some(d) => d.join(self.name),
            None => Path::new(&format!(
                "{}/target/unrealplugin-{}/{}",
                std::env::var("CARGO_MANIFEST_DIR")?,
                std::env::var("TARGET")?,
                self.name
            )).to_path_buf(),
        };
        std::fs::create_dir_all(&outdir).unwrap();

        write_only_if_changed(
            &outdir.join(format!("{}.uplugin", self.name)),
            || {
                let modules = self.modules.iter().collect::<Vec<_>>();
                self.write_plugin(modules.as_slice(), &self.plugin_deps)
            },
        )?;

        std::fs::create_dir_all(outdir.join("Resources")).unwrap();
        let icon_file = outdir.join("Resources/Icon128.png");
        if !Path::new(&icon_file).exists() {
            let icon_file = File::create(icon_file).unwrap();
            Self::write_icon(self.icon, icon_file).unwrap();
        }

        let num_modules = self.modules.len();
        for module in self.modules {
            let mut moduledir = outdir.join("Source");
            if module.name != self.name || num_modules > 1 {
                moduledir = moduledir.join(module.name);
            }
            std::fs::create_dir_all(&moduledir).expect("failed to create output directory");

            write_only_if_changed(
                &moduledir.join(format!("{}.build.cs", module.name)),
                || {
                    Self::write_build(
                        module.external_dylibs,
                        &module.name,
                        module.pub_dep_mods,
                        module.priv_dep_mods,
                        module.pub_include_paths,
                        module.priv_include_paths,
                        module.priv_defs,
                    )
                },
            )
            .unwrap();

            let source_code = module.sources;//.map(|f| f(self.name, &module.name, lib_name.as_str())).transpose()?;

            if !module.external_dylibs.is_empty() {
                if let Some(android) = module.android.as_ref() {
                    let base_apl_file = File::create(moduledir.join("BaseAPL.xml")).unwrap();
                    Self::write_base_apl(android.permissions, module.external_dylibs, base_apl_file).unwrap();
                }
            }

            std::fs::create_dir_all(moduledir.join("Private")).unwrap();
            std::fs::create_dir_all(moduledir.join("Public")).unwrap();

            fn get_default_module<'a>(module_filename: &'a str, module_name: &'a str) -> Result<(&'a str, Vec<CppItem>)> {
                Ok((module_filename, {
                    let name = module_name;
                    vec![CppItem::Header(CppHeader {
                        is_pub: true,
                        contents: {
                            #[derive(Template)]
                            #[template(path = "Module.h.jinja", escape = "none")]
                            struct Template<'a> { name: &'a str }
                            Template { name }.render()?
                        },
                    }),
                    CppItem::Source(CppSource {
                        contents: {
                            #[derive(Template)]
                            #[template(path = "Module.cpp.jinja", escape = "none")]
                            struct Template<'a> { filename: &'a str, name: &'a str, }
                            Template { filename: module_filename, name }.render()?
                        }
                    })]
                }))
            }

            let default_module_filename = format!("{}Module", module.name);
            let sources = match source_code {
                ModuleCppSources::None => vec![get_default_module(&default_module_filename, module.name)?],
                ModuleCppSources::WithDefaultModule(mut items) => {
                    items.push(get_default_module(&default_module_filename, module.name)?);
                    items
                }
                ModuleCppSources::WithoutDefaultModule(items) => items
            };

            for (name, files) in sources {
                for item in files {
                    let (is_pub, contents, ext) = match item {
                        CppItem::Header(CppHeader { is_pub, contents }) => (is_pub, contents, "h"),
                        CppItem::Source(CppSource { contents }) => (false, contents, "cpp"),
                    };
                    let filename = format!("{}.{}", name, ext);
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(moduledir.join(if is_pub { "Public" } else { "Private" }).join(&filename))
                        .expect(format!("failed to open {}", filename).as_str());
                    write!(file, "{}", contents).unwrap();
                }
            }

            let api_out_path = moduledir.join("Private");
            if !api_out_path.exists() {
                std::fs::create_dir_all(&api_out_path)
                    .expect("failed to create api outpath");
            }

            let fwds_out_path = moduledir.join("Public");
            if !fwds_out_path.exists() {
                std::fs::create_dir_all(&fwds_out_path)
                    .expect("failed to create fwds outpath");
            }

            let classes_h_out_path = moduledir.join("Public");
            if !classes_h_out_path.exists() {
                std::fs::create_dir_all(&fwds_out_path)
                    .expect("failed to create classes outpath");
            }

            let classes_cpp_out_path = moduledir.join("Private");
            if !classes_cpp_out_path.exists() {
                std::fs::create_dir_all(&fwds_out_path)
                    .expect("failed to create classes outpath");
            }
        }

        Ok(())
    }
}