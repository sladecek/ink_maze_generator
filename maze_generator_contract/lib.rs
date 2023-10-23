#![cfg_attr(not(feature = "std"), no_std, no_main)]


#[ink::contract]
mod maze_generator_contract {
    use ink::prelude::vec::Vec;


    fn lcg_next(state: &mut u32, modulo: u32) -> u32 {
        *state = (((*state as u64) * 48271u64) % 0x7fffffffu64) as u32;
        *state % modulo
    }

    #[ink(storage)]
    pub struct MazeGeneratorContract {
        opened_doors: Vec<u8>,
    }

    struct MazeBuilder {
        width: i32,
        height: i32,
        visited_rooms: Vec<bool>,
        stack: Vec<i32>,
        random_state: u32,
        opened_doors: Vec<bool>,
    }

    impl MazeBuilder {
        pub fn new(width: i32, height: i32,
                   random_state: u32) -> Self {
            let room_count = (width * height) as usize;
            let mut visited_rooms: Vec<bool> = Vec::with_capacity(room_count);
            visited_rooms.resize(room_count, false);
            let mut opened_doors: Vec<bool> = Vec::with_capacity(room_count * 2);
            opened_doors.resize(room_count * 2, false);
            Self {
                width,
                height,
                visited_rooms,
                stack: Vec::new(),
                random_state,
                opened_doors,
            }
        }

        pub fn build(&mut self) -> Vec<u8> {
            self.stack = Vec::new();
            let start_room: i32 = 0;
            let target_room: i32 = self.height * self.width - 1 as i32;
            self.visit_room(start_room);
            while !self.stack.is_empty() {
                let room = *self.stack.last().unwrap();
                if room == target_room {
                    // fill gaps
                    //ink::env::debug_println!("Reached target room");
                    self.stack.pop();
                    continue;
                }
                let candidates = self.find_all_possible_next_rooms(room);
                if candidates.is_empty() {
                    // backtrace - no way to go
                    //ink::env::debug_println!("Backtrace - no way to go");
                    self.stack.pop();
                    continue;
                }
                // select next room
                let mut choice = 0;
                if candidates.len() > 1 {
                    choice = lcg_next(&mut self.random_state, candidates.len() as u32);
                }
                let wall = candidates[choice as usize];
                self.opened_doors[wall as usize] = true;
                self.visit_room(self.get_room_behind_wall(room, wall));
            }

            self.pack_result()
        }

        fn wall_count(&self) -> i32 {
            self.width * self.height * 2
        }

        fn pack_result(&self) -> Vec<u8> {
            let wall_count = self.wall_count();
            let bytes: usize = ((wall_count + 7) / 8) as usize;
            let mut result: Vec<u8> = Vec::with_capacity(bytes);
            result.resize(bytes, 0u8);
            for i in 0..wall_count {
                if !self.opened_doors[i as usize] {
                    let mask: u8 = 1 << (i % 8);
                    result[(i / 8) as usize] |= mask;
                }
            }
            result
        }

        fn get_room_behind_wall(&self, room: i32, wall: i32) -> i32 {
            let result = {
                if wall == 2 * room {
                    room + 1
                } else if wall == 2 * room + 1 {
                    room + self.width
                } else if wall == 2 * room - 2 {
                    room - 1
                } else {
                    room - self.width
                }
            };
            //ink::env::debug_println!("Get room behind wall room {} wall {} result {}", room, wall, result);
            result
        }

        fn find_all_possible_next_rooms(&self, room: i32) -> Vec<i32> {
            let mut candidates = Vec::new();
            for wall in self.get_walls(room) {
                if !self.opened_doors[wall as usize] {
                    let other_room = self.get_room_behind_wall(room, wall);
                    if other_room < self.width * self.height {
                        if !self.visited_rooms[other_room as usize] {
                            candidates.push(wall);
                        }
                    }
                }
            }
            candidates
        }

        fn visit_room(&mut self, room: i32) {
            ink::env::debug_println!("Visit room {}", room);
            self.visited_rooms[room as usize] = true;
            self.stack.push(room);
        }

        fn get_walls(&self, room: i32) -> Vec<i32> {
            let mut result: Vec<i32> = Vec::new();

            let y = room / self.width;
            let x = room % self.width;

            if x > 0 {
                result.push(2 * room - 2);
            }
            if x < self.width as i32 - 1 {
                result.push(2 * room);
            }
            if y > 0 {
                result.push(2 * room - 2 * self.width + 1);
            }
            if y < self.width as i32 - 1 {
                result.push(2 * room + 1);
            }
            //ink::env::debug_println!("get_walls room={} result={:?}", room, result);
            result
        }
    }

    impl MazeGeneratorContract {
        #[ink(constructor)]
        pub fn new(width: i32, height: i32, random_state: u32) -> Self {
            Self { opened_doors: (MazeBuilder::new(width, height, random_state)).build() }
        }

        /// Simply returns the computed maze
        #[ink(message)]
        pub fn get(&self) -> Vec<u8> {
            self.opened_doors.clone()
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn it_works() {
            let maze_generator_contract = MazeGeneratorContract::new(3, 2, 0);
            assert_eq!(maze_generator_contract.get(), vec![146u8, 15u8]);
        }

        #[ink::test]
        fn it_works_with_different_randomness() {
            let maze_generator_contract = MazeGeneratorContract::new(3, 2, 0x123456);
            assert_eq!(maze_generator_contract.get(), vec![178u8, 14u8]);
        }

        #[ink::test]
        fn lcg_works() {
            let mut state: u32 = 1;
            let r1 = lcg_next(&mut state, 6);
            assert_eq!(state, 48271u32);
            assert_eq!(r1, 1u32);
            let r2 = lcg_next(&mut state, 6);
            assert_eq!(r2, 0u32);
        }
    }
}
