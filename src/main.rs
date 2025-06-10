use geo_tiler::{generate_grid, Tile};
fn main() {
    let grid: Vec<Tile> = generate_grid(10);

    let mut count = 0;
    for tile in grid {
        count += 1;
        println!("{}", tile);
    }
    println!("count: {}", count);
}