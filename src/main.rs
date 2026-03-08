use rstrace::color;
fn main() {
    // Leave two lines between the execution and the printing of the main
    println! {"\n------------------------------------------------------\n"};

    // Implement the first color
    let our_color: color::Color = color::Color {
        r: 1.0,
        g: 2.0,
        b: 3.0,
    };
    println! {"Hey! Here's our fist color: {:?} \n", our_color };

    // Implement the second color
    let another_color: color::Color = color::Color {
        r: 4.0,
        g: 5.0,
        b: 6.0,
    };
    println! {"Hey! Here's another color : {:?} \n", another_color };

    // Sum the color!
    let sum: color::Color = our_color + another_color;
    println! {"Hey! Here the sum!?!?     : {:?} \n", sum };

    // Multiply and divide the second color by scalars!
    let lambda = - 0.5;
    let product = lambda * another_color;
    println! {"Hey! Here the scalar * another_color product : {:?} \n", product };
    let product = another_color * lambda;
    println! {"Hey! Here the another_color * scalar product  : {:?} \n", product };

    let lambda = - 0.25;
    let division = product / lambda;
    println!("Hey! Here the quotient of division : {:?} \n", division);
}
