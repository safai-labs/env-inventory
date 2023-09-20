mod other;

env_inventory::register!("HELLO");

fn main() {
    println!("ENV VARS\n{:?}", env_inventory::list_all_vars());
    env_inventory::load_and_validate_env_vars(&[], "none").unwrap();
}
