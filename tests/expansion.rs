use gds_uim::el;

// Deliberately retain Rust's default recursion limit. This builder is move-only.
#[derive(Default)]
struct Node {
    children: Vec<Node>,
    styles: usize,
}

fn node() -> Node {
    Node::default()
}

impl Node {
    fn styled(mut self) -> Self {
        self.styles += 1;
        self
    }

    fn child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }
}

#[test]
fn nested_attributes_do_not_accumulate_on_the_child_expansion_stack() {
    let tree = el! {
        #node { styled; styled; } [
            #node { styled; styled; } [
                #node { styled; styled; } [
                    #node { styled; styled; } [
                        #node { styled; styled; } [
                            #node { styled; styled; } [
                                #node { styled; styled; } [
                                    #node { styled; styled; } [
                                        #node { styled; styled; } [
                                            #node { styled; styled; } [
                                                #node { styled; styled; } [
                                                    #node { styled; styled; } [
                                                        #node { styled; styled; } [
                                                            #node { styled; styled; } [
                                                                #node { styled; styled; } [
                                                                    #node { styled; styled; } [
                                                                        #node { styled; styled; } [
                                                                            #node { styled; styled; } [
                                                                                #node { styled; styled; } [
                                                                                    #node { styled; styled; } [
                                                                                        #node { styled; styled; } [
                                                                                            #node { styled; styled; } [
                                                                                                #node { styled; styled; } [
                                                                                                    #node { styled; styled; } [
                                                                                                        #node { styled; styled; } [
                                                                                                            #node { styled; styled; } [
                                                                                                                #node { styled; styled; } [
                                                                                                                    #node { styled; styled; } [
                                                                                                                        #node { styled; styled; } [
                                                                                                                            #node { styled; styled; } [
                                                                                                                                #node { styled; styled; } [
                                                                                                                                    #node { styled; styled; }
                                                                                                                                ]
                                                                                                                            ]
                                                                                                                        ]
                                                                                                                    ]
                                                                                                                ]
                                                                                                            ]
                                                                                                        ]
                                                                                                    ]
                                                                                                ]
                                                                                            ]
                                                                                        ]
                                                                                    ]
                                                                                ]
                                                                            ]
                                                                        ]
                                                                    ]
                                                                ]
                                                            ]
                                                        ]
                                                    ]
                                                ]
                                            ]
                                        ]
                                    ]
                                ]
                            ]
                        ]
                    ]
                ]
            ]
        ]
    };

    let mut depth = 0;
    let mut current = &tree;
    loop {
        depth += 1;
        assert_eq!(current.styles, 2);
        match current.children.as_slice() {
            [] => break,
            [child] => current = child,
            _ => panic!("each level has at most one child"),
        }
    }
    assert_eq!(depth, 32);
}
