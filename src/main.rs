use sabratha::connect;

fn main() {
    let connection = connect();
    println!("{:#?}", connection);
}
