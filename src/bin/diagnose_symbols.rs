use colored::Colorize;

fn main() {
    println!("{}", "SlopChop Symbol Diagnostics".bold().cyan());
    println!("Run this, verify which symbols render correctly, and report back.\n");

    let categories = [
        ("ASCII / Safe", vec![
            (">", "Greater Than"),
            ("-", "Dash"),
            ("*", "Asterisk"),
            ("[OK]", "Brackets"),
        ]),
        ("Checkmarks", vec![
            ("�", "U+2713 Check Mark"),
            ("?", "U+2714 Heavy Check Mark"),
            ("?", "U+2705 White Heavy Check Mark"),
            ("�", "U+221A Square Root"),
            ("?", "U+237B Not Check Mark"),
            ("??", "U+1F5F9 Ballot Box with Check"),
        ]),
        ("Crosses / Errors", vec![
            ("?", "U+2717 Ballot X"),
            ("?", "U+2718 Heavy Ballot X"),
            ("?", "U+274C Cross Mark"),
            ("x", "U+00D7 Multiplication Sign"),
            ("?", "U+2612 Ballot Box with X"),
            ("?", "U+2716 Heavy Multiplication X"),
        ]),
        ("Arrows / Pointers", vec![
            ("", "U+2192 Right Arrow"),
            ("?", "U+279C Heavy Round-Tipped Right Arrow"),
            ("?", "U+27F9 Long Double Right Arrow"),
            ("?", "U+25B6 Black Right-Pointing Triangle"),
            ("?", "U+27A4 Black Rightwards Arrowhead"),
            ("?", "U+276F Heavy Right-Pointing Angle Quotation Mark"),
            ("�", "U+00BB Right-Pointing Double Angle Quotation Mark"),
            ("?", "U+2794 Heavy Wide-Headed Rightwards Arrow"),
        ]),
        ("Icons / Status", vec![
            ("?", "U+2699 Gear"),
            ("??", "U+2699 + VS16 Gear Emoji"),
            ("??", "U+1F527 Wrench"),
            ("??", "U+1F6E0 Hammer and Wrench"),
            ("??", "U+1F6E1 Shield"),
            ("???", "U+1F6E1 + VS16 Shield Emoji"),
            ("?", "U+26A0 Warning Sign"),
            ("?", "U+26A1 High Voltage"),
            ("?", "U+2728 Sparkles"),
            ("??", "U+1F680 Rocket"),
            ("??", "U+1F4E6 Package"),
            ("??", "U+1F9F6 Yarn"),
        ]),
        ("Geometric / Bullets", vec![
            ("", "U+2022 Bullet"),
            ("?", "U+25CF Black Circle"),
            ("	", "U+25CB White Circle"),
            ("?", "U+25AA Black Small Square"),
            ("?", "U+25AB White Small Square"),
            ("?", "U+25C6 Black Diamond"),
        ]),
    ];

    for (category, items) in categories {
        println!("{}:", category.yellow().bold());
        for (symbol, desc) in items {
            println!("  {symbol:<4}  {desc}");
        }
        println!();
    }
}