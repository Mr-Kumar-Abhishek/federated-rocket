use clap::Args;

#[derive(Args)]
pub struct InfoArgs {
    /// Path to .ork or .rkt file
    pub file: String,

    /// Show detailed component information
    #[arg(short, long)]
    pub detailed: bool,

    /// JSON output
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: InfoArgs) -> anyhow::Result<()> {
    let path = std::path::Path::new(&args.file);
    let tree = federated_rocket_fileio::format_detect::load_rocket_file(path)?;

    if args.json {
        // Serialize component tree to JSON
        println!("{}", serde_json::to_string_pretty(&tree)?);
    } else {
        println!("\n=== Rocket Design Info ===");
        println!("File: {}", args.file);
        println!("Components: {}", tree.component_count());

        if let Some(root_key) = tree.root() {
            if let Some(node) = tree.get(root_key) {
                println!("Name: {}", node.component.name());
                println!("Type: {}", node.component.component_type());
            }
        }

        // List all components
        println!("\n=== Component List ===");
        for (key, node) in tree.iter() {
            let depth = tree.depth(key);
            let indent = "  ".repeat(depth);
            println!(
                "{}{} ({})",
                indent,
                node.component.name(),
                node.component.component_type()
            );

            if args.detailed {
                if let Some(parent_key) = tree.parent(key) {
                    if let Some(parent_node) = tree.get(parent_key) {
                        println!("{}  Parent: {}", indent, parent_node.component.name());
                    }
                }
                println!("{}  Children: {}", indent, node.children.len());
            }
        }
    }

    Ok(())
}
