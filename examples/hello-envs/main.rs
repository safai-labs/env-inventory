mod other;

env_inventory::register!("HELLO");

fn main() {
    println!("ENV VARS\n{:?}", env_inventory::list_all_vars());
    env_inventory::load_and_validate_env_vars(&["./examples/hello-envs/hello-world.toml"], "env").unwrap();
	// show environment variables
	println!("{:?}", std::env::var("HELLO").unwrap());
	// the following will panic if WORLD is not set
	// you can set it in the shell with `export WORLD=world`
	// or uncomment it in hello-world.toml
	println!("{:?}", std::env::var("WORLD").unwrap());
}
