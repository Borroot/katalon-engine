/// Return all of the best moves and the evaluation.
pub fn bestmoves(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> (Result<(eval::Eval, Vec<(u8, u8)>), ()>, search::Stats) {
    // TODO give without evaluation, so if there is just one possible move we can immediately return
    let now = std::time::Instant::now();

    let mut minmax = Minmax {
        recv_timeout,
        rootcount: node.movecount(),
        rootplayer: node.onturn(),
        // TODO make adaptive to the movecount
        table: table::Table::from_gb(1.0),
        stats: search::Stats::new(),
    };

    let mut bestmoves: Vec<(u8, u8)> = Vec::new();
    let mut max = eval::Eval::MIN;

    let moves = node.moves();
    // TODO sort the moves

    // TODO add parallelization
    // TODO add iterative deepening and null window search
    for &(square, cell) in &moves {
        let mut child = node.clone();
        child.play(square, cell);

        let alpha = eval::Eval::MIN;
        let beta = eval::Eval::MAX;

        // TODO reuse improved alpha (beta does not change here)
        let value = negamax(&child, alpha, beta, &mut minmax);

        if value.is_err() {
            minmax.stats.table = minmax.table.stats();
            minmax.stats.time = now.elapsed();

            return (Err(()), minmax.stats);
        }

        let value = -value.unwrap();
        if value > max {
            max = value;
            bestmoves.clear();
            bestmoves.push((square, cell));
        } else if value == max {
            bestmoves.push((square, cell));
        }
    }

    minmax.stats.table = minmax.table.stats();
    minmax.stats.time = now.elapsed();

    //// reset table and time
    //let now = std::time::Instant::now();
    //minmax.table = table::Table::from_gb(1.0);
    //println!("------------------------------------------------------ {}ms", now.elapsed().as_millis());

    //let mut wmax = eval::Eval::MAX.n;
    //let mut wmin = eval::Eval::MIN.n;
    //let mut med = 0;
    //let mut count = 0;

    //loop {
    //    count += 1;
    //    med = (wmin + wmax) / 2;

    //    let r = negamax(&node, eval::Eval::new(med - 1), eval::Eval::new(med + 1), &mut minmax);
    //    if r.is_err() {
    //        println!("TIMEOUT");
    //        break;
    //    }
    //    let r = r.unwrap().n;

    //    println!("------------------------------------------------------ {}ms", now.elapsed().as_millis());
    //    println!("min = {}, med = {}, max = {}, r = {}", wmin, med, wmax, r);
    //    println!("min = {}, med = {}, max = {}, r = {}", eval::Eval::new(wmin), eval::Eval::new(med), eval::Eval::new(wmax), eval::Eval::new(r));

    //    if med == r {
    //        break;
    //    } else if r < med {
    //        wmax = r;
    //    } else {
    //        wmin = r;
    //    }
    //}
    //println!("med = {}, loops = {}", med, count);
    //println!("evaluation: {}, {}ms", eval::Eval::new(med), now.elapsed().as_millis());

    //// reset table and time
    //let now = std::time::Instant::now();
    //minmax.table = table::Table::from_gb(1.0);
    //println!("------------------------------------------------------ {}ms", now.elapsed().as_millis());

    //let mut wmax = eval::Eval::MAX.n;
    //let mut wmin = eval::Eval::MIN.n;
    //let mut guess = 0;
    //let mut count = 0;

    //while wmin < wmax {
    //    count += 1;
    //    let beta = std::cmp::max(guess, wmin + 1);

    //    let tmp_guess = negamax(&node, eval::Eval::new(beta - 1), eval::Eval::new(beta), &mut minmax);
    //    if tmp_guess.is_err() {
    //        println!("TIMEOUT");
    //        break;
    //    }
    //    guess = tmp_guess.unwrap().n;

    //    //println!("------------------------------------------------------ {}ms", now.elapsed().as_millis());
    //    //println!("min = {}, max = {}, g = {}", wmin, wmax, guess);
    //    //println!("min = {}, max = {}, g = {}", eval::Eval::new(wmin), eval::Eval::new(wmax), eval::Eval::new(guess));

    //    if guess < beta {
    //        wmax = guess;
    //    } else {
    //        wmin = guess;
    //    }
    //}
    //println!("guess = {}, loops = {}", guess, count);
    //println!("evaluation: {}, {}ms", eval::Eval::new(guess), now.elapsed().as_millis());

    (Ok((max, bestmoves)), minmax.stats)
}
