
fn main(x: Field, y: pub Field, z: Field) -> pub Field {
    let block = [x,y];
    let test = &mut block;
    test[x] = z;
    z + test[x]
}
