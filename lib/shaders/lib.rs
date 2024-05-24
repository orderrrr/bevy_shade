use cgmath::Vector3;


#[allow(dead_code)]
fn get_child_pos(parent_pos: &Vector3<u32>, child_rel_pos: &Vector3<u32>, parent_depth: u32) -> Vector3<u32> {
    let d: u32 = 1 << parent_depth;
    
    let parent_offset = parent_pos * d;

    println!("+++++++++++++++++");
    println!("depth: {:?}", parent_depth);
    println!("parent depth: {:?}", d);
    println!("parent pos: {:?}", parent_pos);
    println!("parent offset: {:?}", parent_offset);
    println!("child_rel_pos: {:?}", child_rel_pos);

    parent_offset + child_rel_pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_uniqueness_at_depth_1() {
        let parent_depth = 1;
        let mut positions = HashSet::new();

        for parent_x in 0..1<<parent_depth {
            for parent_y in 0..1<<parent_depth {
                for parent_z in 0..1<<parent_depth {
                    let parent_pos = Vector3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = Vector3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos, parent_depth);
                                assert!(child_pos.x < 4);
                                assert!(child_pos.y < 4);
                                assert!(child_pos.z < 4);
                                assert!(positions.insert(child_pos));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(positions.len(), 64);
    }

    #[test]
    fn test_uniqueness_at_depth_2() {
        let parent_depth = 2;
        let mut positions = HashSet::new();

        for parent_x in 0..1<<parent_depth {
            for parent_y in 0..1<<parent_depth {
                for parent_z in 0..1<<parent_depth {

                    let parent_pos = Vector3::new(parent_x, parent_y, parent_z);

                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = Vector3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos, parent_depth);
                                assert!(child_pos.x < (1<<(parent_depth + 1)));
                                assert!(child_pos.y < (1<<(parent_depth + 1)));
                                assert!(child_pos.z < (1<<(parent_depth + 1)));
                                assert!(positions.insert(child_pos));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(positions.len(), 512);
    }

    #[test]
    fn test_uniqueness_at_depth_3() {
        let parent_depth = 3;
        let mut positions = HashSet::new();

        for parent_x in 0..1<<parent_depth {
            for parent_y in 0..1<<parent_depth {
                for parent_z in 0..1<<parent_depth {
                    let parent_pos = Vector3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = Vector3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos, parent_depth);
                                assert!(positions.insert(child_pos));
                                assert!(child_pos.x < (1<<(parent_depth) + 1));
                                assert!(child_pos.y < (1<<(parent_depth) + 1));
                                assert!(child_pos.z < (1<<(parent_depth) + 1));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(positions.len(), 4096);
    }
}
