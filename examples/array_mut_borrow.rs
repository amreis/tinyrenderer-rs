fn main() {
    let mut c = [30u8, 40, 60];

    println!("For starters, c = {:?}", c);
    for elem in &mut c {
        *elem += 2;
    }
    print!("After for elem in &mut c: ");
    println!("{:?}", c);

    for ref mut elem in c.into_iter() {
        *elem += 3;
    }
    print!("After for ref mut elem in c: ");
    println!("{:?}", c);

    for elem in c.each_mut() {
        *elem += 10;
    }
    print!("After for elem in c.each_mut(): ");
    println!("{:?}", c);

}