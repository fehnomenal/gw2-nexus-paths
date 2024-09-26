use paths_data::markers::MarkerCategoryTree;
use paths_types::{MarkerCategory, Point3, TrailDescription};

#[derive(Debug)]
pub struct ActiveMarkerCategories<'a, C: Clone> {
    categories: Vec<&'a MarkerCategory<C>>,
    active_map_id: u32,
    active_points_of_interest: Vec<ActivePointOfInterest>,
    active_trails: Vec<ActiveTrail<C>>,
}

impl<'a, C: Clone> ActiveMarkerCategories<'a, C> {
    pub fn new() -> Self {
        Self {
            categories: vec![],
            active_map_id: 0,
            active_points_of_interest: vec![],
            active_trails: vec![],
        }
    }

    pub fn read_from_tree(&mut self, tree: &'a MarkerCategoryTree<C>) {
        self.categories = tree
            .tree
            .root()
            .unwrap()
            .traverse_pre_order()
            .filter_map(|n| {
                let category = n.data();

                let is_active = *category.is_active.borrow();
                let has_pois = !category.points_of_interest.is_empty();
                let has_trails = !category.trails.is_empty();

                if is_active && (has_pois || has_trails) {
                    Some(category)
                } else {
                    None
                }
            })
            .collect();

        self.set_active_map(self.active_map_id);
    }

    pub fn set_active_map(&mut self, map_id: u32) {
        self.active_map_id = map_id;
        self.active_points_of_interest.clear();
        self.active_trails.clear();

        for category in self.categories.iter() {
            if !category.points_of_interest.is_empty() {
                // TODO
            }

            let trails = category.trails.to_vec();
            for trail_description in trails.iter() {
                if let TrailDescription::Loaded(loaded) = trail_description {
                    if loaded.trail.map_id == map_id {
                        self.active_trails.push(ActiveTrail {
                            trail_width: category.trail_width,
                            trail_color: category.trail_color.clone(),
                            points: loaded.trail.points.clone(),
                        });
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ActiveTrail<C: Clone> {
    pub trail_width: Option<f32>,
    pub trail_color: Option<C>,
    pub points: Vec<Point3>,
}

#[derive(Debug)]
pub struct ActivePointOfInterest {
    // TODO
}
