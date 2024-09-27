use std::collections::HashMap;

use paths_data::markers::MarkerCategoryTree;
use paths_types::{MarkerCategory, Point3};

#[derive(Debug)]
pub struct ActiveMarkerCategories<'a, C> {
    categories: Vec<&'a MarkerCategory<C>>,
    pub active_map_id: u32,
    all_points_of_interest: HashMap<u32, Vec<ActivePointOfInterest>>,
    pub all_trails: HashMap<u32, Vec<ActiveTrail<'a, C>>>,
    empty_poi_vec: Vec<ActivePointOfInterest>,
    empty_trail_vec: Vec<ActiveTrail<'a, C>>,
}

impl<'a, C> ActiveMarkerCategories<'a, C> {
    pub fn new() -> Self {
        Self {
            categories: vec![],
            active_map_id: 0,
            all_points_of_interest: HashMap::default(),
            all_trails: HashMap::default(),
            empty_poi_vec: vec![],
            empty_trail_vec: vec![],
        }
    }

    pub fn read_from_tree(&mut self, tree: &'a MarkerCategoryTree<C>) {
        self.all_points_of_interest.clear();
        self.all_trails.clear();

        self.categories = tree
            .tree
            .root()
            .unwrap()
            .traverse_pre_order()
            .filter_map(|n| {
                let category = n.data();

                if (*category.is_active.borrow())
                    && ((!category.points_of_interest.is_empty()) || (!category.trails.is_empty()))
                {
                    Some(category)
                } else {
                    None
                }
            })
            .collect();

        for category in &self.categories {
            for _poi in &category.points_of_interest {
                // TODO
            }

            for trail in &category.trails {
                self.all_trails
                    .entry(trail.map_id)
                    .or_default()
                    .push(ActiveTrail {
                        #[cfg(debug_assertions)]
                        id: &category.identifier,
                        trail_width: &category.trail_width,
                        trail_color: &category.trail_color,
                        points: &trail.points,
                    });
            }
        }

        #[cfg(debug_assertions)]
        {
            eprintln!("loaded active marker categories");
            eprintln!(
                "points of interest: {}",
                self.all_points_of_interest
                    .values()
                    .map(|points_of_interest| points_of_interest.len())
                    .sum::<usize>(),
            );
            eprintln!(
                "trails: {}",
                self.all_trails
                    .values()
                    .map(|trails| trails.len())
                    .sum::<usize>(),
            );
            eprintln!(
                "total points: {}",
                self.all_trails
                    .values()
                    .flat_map(|trails| trails.iter().map(|trail| trail.points.len()))
                    .sum::<usize>(),
            );
        }

        self.set_active_map(self.active_map_id);
    }

    pub fn set_active_map(&mut self, map_id: u32) {
        self.active_map_id = map_id;

        #[cfg(debug_assertions)]
        {
            eprintln!("changed active map to {map_id}");

            const MAX: usize = 10;

            let active_points_of_interest = self.active_points_of_interest();

            eprintln!(
                "active points of interest ({}): {:?}",
                active_points_of_interest.len(),
                active_points_of_interest
                    .iter()
                    .take(MAX)
                    .collect::<Vec<_>>(),
            );

            let active_trails = self.active_trails();

            eprintln!(
                "active trails ({}): {:?}",
                active_trails.len(),
                active_trails
                    .iter()
                    .map(|trail| trail.id.join("."))
                    .take(MAX)
                    .collect::<Vec<_>>(),
            );
            eprintln!(
                "active trail points: {}",
                active_trails
                    .iter()
                    .map(|trail| trail.points.len())
                    .sum::<usize>(),
            )
        }
    }

    pub fn active_points_of_interest(&self) -> &Vec<ActivePointOfInterest> {
        self.all_points_of_interest
            .get(&self.active_map_id)
            .unwrap_or_else(|| &self.empty_poi_vec)
    }

    pub fn active_trails(&self) -> &Vec<ActiveTrail<C>> {
        self.all_trails
            .get(&self.active_map_id)
            .unwrap_or_else(|| &self.empty_trail_vec)
    }
}

#[derive(Debug)]
pub struct ActiveTrail<'a, C> {
    #[cfg(debug_assertions)]
    pub id: &'a Vec<String>,
    pub trail_width: &'a Option<f32>,
    pub trail_color: &'a Option<C>,
    pub points: &'a Vec<Point3>,
}

#[derive(Debug)]
pub struct ActivePointOfInterest {
    // TODO
}
