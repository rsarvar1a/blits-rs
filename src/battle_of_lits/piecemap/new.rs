use super::*;

impl PieceMap {
    /// Creates a new PieceMap.
    pub fn new() -> PieceMap {
        // man just give us placement new already
        let forward = unsafe { 
            let mut tetrominos: Box<MaybeUninit<[Tetromino; NUM_PIECES]>> = Box::new_zeroed();
            let mut i = 0;

            Tile::all().iter().for_each(|kind| {
                (0..10).cartesian_product(0..10).map(|(row, col)| Coord { row, col }).for_each(|anchor| {
                    Tetromino::identity(*kind, &anchor).enumerate().iter().for_each(|isomorph| {
                        if isomorph.in_bounds() {
                            *tetrominos.assume_init_mut().get_unchecked_mut(i) = *isomorph;
                            i += 1;
                        }
                    });
                });
            });

            tetrominos.assume_init()
        };

        let reverse = forward.iter().enumerate().map(|(i, piece): (usize, &Tetromino)| (piece.real_coords(), i)).collect::<HashMap<[OffsetCoord; 4], usize>>();
        let mut associations = vec![vec![Interaction::Conflicting; NUM_PIECES]; NUM_PIECES];

        for i in 0..NUM_PIECES {
            for j in (i + 1)..NUM_PIECES {
                let [lhs, rhs] = [forward[i], forward[j]];
                let [l_coords, r_coords] = [lhs, rhs].map(|p: Tetromino| p.real_coords().into_iter().collect::<std::collections::HashSet<OffsetCoord>>());

                // 1. do the pieces intersect?
                if l_coords.intersection(&r_coords).cloned().collect::<BTreeSet<_>>().len() > 0 {
                    associations[i][j] = Interaction::Conflicting;
                    continue;
                }

                // 2. do the pieces have no neighbouring tiles?
                if ! l_coords.iter().any(|l| {
                    r_coords.iter().any(|r: &OffsetCoord| r.neighbours(*l))
                }) {
                    associations[i][j] = Interaction::Neutral;
                    continue;
                }

                // 3. are the pieces adjacent and of the same type?
                if lhs.kind == rhs.kind {
                    associations[i][j] = Interaction::Conflicting;
                    continue;
                }

                // 4. do these two pieces alone violate the foursquare rule?
                let cover = l_coords.union(&r_coords).cloned().collect::<std::collections::HashSet<_>>();
                if cover.iter().any(|c| {
                    cover.contains(&OffsetCoord { rows: c.rows + 1, cols: c.cols })
                        && cover.contains(&OffsetCoord { rows: c.rows, cols: c.cols + 1 })
                        && cover.contains(&OffsetCoord { rows: c.rows + 1, cols: c.cols + 1 })
                }) {
                    associations[i][j] = Interaction::Conflicting;
                    continue;
                }

                associations[i][j] = Interaction::Adjacent;
            }
        }

        let associations_specific: Box<[[MoveSet; 3]; NUM_PIECES]> = unsafe {
            let mut specific: Box<MaybeUninit<[[MoveSet; 3]; NUM_PIECES]>> = Box::new_zeroed();
            for idx in 0..NUM_PIECES {
                for int in [Interaction::Conflicting, Interaction::Neutral, Interaction::Adjacent] {
                    let set: MoveSet = (0..NUM_PIECES).filter(|&p| associations[idx.min(p)][idx.max(p)] == int).collect();
                    *specific.assume_init_mut().get_unchecked_mut(idx).get_unchecked_mut(int as usize) = set;
                }
            }
            specific.assume_init()
        };

        // man just give us placement new already
        let neighbours = unsafe {
            let mut neighbours: Box<MaybeUninit<[CoordSet; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *neighbours.assume_init_mut().get_unchecked_mut(idx) = forward[idx].neighbours();
            });
            neighbours.assume_init()
        };

        let selfs = unsafe {
            let mut selfs: Box<MaybeUninit<[CoordSet; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *selfs.assume_init_mut().get_unchecked_mut(idx) = CoordSet::from_iter(forward[idx].real_coords_lazy().map(|c| c.coerce()));
            });
            selfs.assume_init()
        };

        let coord_neighbours = unsafe {
            let mut neighbours: Box<MaybeUninit<[CoordSet; 100]>> = Box::new_zeroed();
            (0..10).cartesian_product(0..10).for_each(|(row, col)| {
                let idx = row * BOARD_SIZE + col;
                let c = Coord { row, col };
                let mut set = CoordSet::default();
                ORTHOGONAL_OFFSETS.iter().for_each(|offset| {
                    let candidate = c + offset;
                    if candidate.in_bounds_signed() {
                        set.insert(&candidate.coerce());
                    }
                });
                *neighbours.assume_init_mut().get_unchecked_mut(idx) = set;
            });
            neighbours.assume_init()
        };

        let chokepoints = unsafe {
            let mut chokepoints: Box<MaybeUninit<[Vec<Coord>; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *chokepoints.assume_init_mut().get_unchecked_mut(idx) = chokepoints::compute_chokepoints(&forward[idx]);
            });
            chokepoints.assume_init()
        };

        let bridges = unsafe {
            let mut bridges: Box<MaybeUninit<[Vec<(Coord, Coord)>; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *bridges.assume_init_mut().get_unchecked_mut(idx) = bridges::compute_connectivity_bridges(&forward[idx]);
            });
            bridges.assume_init()
        };

        let isolation_potential = unsafe {
            let mut isolation_potential: Box<MaybeUninit<[bool; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *isolation_potential.assume_init_mut().get_unchecked_mut(idx) = isolation::compute_isolation_potential(&forward[idx]);
            });
            isolation_potential.assume_init()
        };

        let connectivity_dependencies = unsafe {
            let mut connectivity_dependencies: Box<MaybeUninit<[MoveSet; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *connectivity_dependencies.assume_init_mut().get_unchecked_mut(idx) = dependencies::compute_connectivity_dependencies(&forward[idx], idx, &*forward);
            });
            connectivity_dependencies.assume_init()
        };

        let isolation_shadows = unsafe {
            let mut isolation_shadows: Box<MaybeUninit<[Vec<(Coord, CoordSet)>; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                *isolation_shadows.assume_init_mut().get_unchecked_mut(idx) = shadows::compute_isolation_shadows(&forward[idx], idx);
            });
            isolation_shadows.assume_init()
        };

        let shadowsets = unsafe {
            let mut shadowsets: Box<MaybeUninit<[CoordSet; NUM_PIECES]>> = Box::new_zeroed();
            (0..NUM_PIECES).for_each(|idx| {
                let mut shadowset = selfs[idx].clone();
                shadowset.union_inplace(&neighbours[idx]);
                *shadowsets.assume_init_mut().get_unchecked_mut(idx) = shadowset;
            });
            shadowsets.assume_init()
        };

        let pieces_by_type = {
            let mut sets = [MoveSet::default(); 4];
            for idx in 0..NUM_PIECES {
                let tile = forward[idx].kind;
                sets[tile as usize].insert(idx);
            }
            sets
        };

        PieceMap {
            forward,
            reverse,
            associations,
            associations_specific,
            coord_neighbours,
            neighbours,
            selfs,
            chokepoints,
            bridges,
            isolation_potential,
            connectivity_dependencies,
            isolation_shadows,
            shadowsets,
            pieces_by_type
        }
    }
}