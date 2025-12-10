#![allow(clippy::print_stdout)]

#[allow(clippy::cognitive_complexity)]
pub fn render_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }

    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    print!("┌");
    for (i, &width) in widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < widths.len() - 1 {
            print!("┬");
        }
    }
    println!("┐");

    print!("│");
    for (i, (header, &width)) in headers.iter().zip(widths.iter()).enumerate() {
        print!(" {header:<width$} ");
        if i < widths.len() - 1 {
            print!("│");
        }
    }
    println!("│");

    print!("├");
    for (i, &width) in widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < widths.len() - 1 {
            print!("┼");
        }
    }
    println!("┤");

    for row in rows {
        print!("│");
        for (i, (cell, &width)) in row.iter().zip(widths.iter()).enumerate() {
            let truncated = if cell.len() > width {
                &cell[..width.saturating_sub(3)]
            } else {
                cell
            };
            print!(" {truncated:<width$} ");
            if i < widths.len() - 1 {
                print!("│");
            }
        }
        println!("│");
    }

    print!("└");
    for (i, &width) in widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < widths.len() - 1 {
            print!("┴");
        }
    }
    println!("┘");
}
