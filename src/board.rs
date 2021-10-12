struct Board {
    // onturn: who is on turn
    // cells: the actual cells on the board

    // stone_count1: stones left for player 1
    // stone_count2: stones left for player 2

    // square_constraint: you can only move in this square
    // previous_move: this piece cannot be taken
    // taken_streak: if last 10 turns only pieces are taken, then its a draw

    // move_count: number of moves made
}

impl Board {
    // new(moves)
    // new()

    // canplay(move)
    // play(move)
    // isfinalmove(move): the other player has no stones left after your turn
    // (you lose) you fill a square (you win), the taken_streak is reached
    // (draw), or the board is full (depends)

    // nbmoves(): returns move_count
    // hash(): small bit representation of the board
}