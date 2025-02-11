# UEPluginGen

## Helps generate Unreal Engine plugin boilerplate.

Example usage:

    // build.rs
    use ueplugingen::*;

    fn main() {
        Builder::new("MyPlugin")
            .module(Module {
                name: "MyPlugin",
                android: None,
                pub_dep_mods: &[],
                priv_dep_mods: &[],
                priv_defs: &[],
                pub_include_paths: &[],
                priv_include_paths: &[],
                whitelist_platforms: &[],
                sources: None,
                ty: HostType::Runtime,
                loading_phase: LoadingPhanse::Default
            })
            .generate(&"myplugin").expect("failed to generate plugin");
    }
