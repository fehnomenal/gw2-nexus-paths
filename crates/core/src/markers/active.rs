use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

#[cfg(debug_assertions)]
use log::trace;
use log_err::LogErrOption;

use crate::{
    markers::MarkerCategoryTreeNode,
    points::Point3,
    settings::{TrailColor, TrailWidth},
};

use super::MarkerCategoryTree;

#[derive(Debug)]
pub struct ActiveMarkerCategories<'a> {
    pub active_category_count: usize,
    current_map_id: u32,
    active_points_of_interest_by_map: HashMap<u32, Vec<ActivePointOfInterest<'a>>>,
    active_trails_by_map: HashMap<u32, Vec<ActiveTrail<'a>>>,
}

impl<'a> ActiveMarkerCategories<'a> {
    pub fn new() -> Self {
        Self {
            active_category_count: 0,
            current_map_id: 0,
            active_points_of_interest_by_map: HashMap::default(),
            active_trails_by_map: HashMap::default(),
        }
    }

    pub fn read_from_tree(&mut self, tree: &'a MarkerCategoryTree) {
        self.active_category_count = 0;
        self.active_points_of_interest_by_map.clear();
        self.active_trails_by_map.clear();

        fn collect_active_categories<'a>(
            parent: &MarkerCategoryTreeNode<'a>,
            parent_is_active: bool,
            parent_trail_color: &TrailColor,
            parent_trail_width: &TrailWidth,
            category_count: &mut usize,
            all_points_of_interest: &mut HashMap<u32, Vec<ActivePointOfInterest>>,
            all_trails: &mut HashMap<u32, Vec<ActiveTrail<'a>>>,
        ) {
            for child in parent.children() {
                let category = child.data();

                let child_is_active = category.is_active.borrow().unwrap_or(parent_is_active);
                let child_trail_color = category
                    .trail_color
                    .borrow()
                    .unwrap_or_else(|| *parent_trail_color);
                let child_trail_width = category
                    .trail_width
                    .borrow()
                    .unwrap_or_else(|| *parent_trail_width);

                if child_is_active
                    && (!category.points_of_interest.is_empty() || !category.trails.is_empty())
                {
                    *category_count += 1;

                    for _poi in &category.points_of_interest {
                        // TODO
                    }

                    let mut hasher = DefaultHasher::new();
                    category.identifier.hash(&mut hasher);
                    let hash = hasher.finish();

                    for trail in &category.trails {
                        all_trails
                            .entry(trail.map_id)
                            .or_default()
                            .push(ActiveTrail {
                                #[cfg(debug_assertions)]
                                id: &category.identifier,
                                hash,
                                trail_width: child_trail_width,
                                trail_color: child_trail_color,
                                points: &trail.points,
                            });
                    }
                }

                collect_active_categories(
                    &child,
                    child_is_active,
                    &child_trail_color,
                    &child_trail_width,
                    category_count,
                    all_points_of_interest,
                    all_trails,
                );
            }
        }

        let root = tree.tree.root().log_unwrap();
        let root_category = root.data();

        collect_active_categories(
            &root,
            false,
            &root_category.trail_color.borrow().log_unwrap(),
            &root_category.trail_width.borrow().log_unwrap(),
            &mut self.active_category_count,
            &mut self.active_points_of_interest_by_map,
            &mut self.active_trails_by_map,
        );

        #[cfg(debug_assertions)]
        {
            trace!("loaded active marker categories");
            trace!(
                "points of interest: {}",
                self.active_points_of_interest_by_map
                    .values()
                    .map(|points_of_interest| points_of_interest.len())
                    .sum::<usize>(),
            );
            trace!(
                "trails: {}",
                self.active_trails_by_map
                    .values()
                    .map(|trails| trails.len())
                    .sum::<usize>(),
            );
            trace!(
                "total points: {}",
                self.active_trails_by_map
                    .values()
                    .flat_map(|trails| trails.iter().map(|trail| trail.points.len()))
                    .sum::<usize>(),
            );
        }

        self.set_current_map(self.current_map_id);
    }

    pub fn set_current_map(&mut self, map_id: u32) {
        self.current_map_id = map_id;

        #[cfg(debug_assertions)]
        {
            trace!("changed current map to {map_id}");

            const MAX: usize = 10;

            let active_points_of_interest = self
                .active_points_of_interest_by_map
                .get(&map_id)
                .map_or_else(|| [].iter(), |v| v.iter())
                .map(|o| o.id.join("."));

            trace!(
                "active points of interest ({}): {:?}",
                active_points_of_interest.len(),
                active_points_of_interest.take(MAX).collect::<Vec<_>>(),
            );

            let active_trails = self
                .active_trails_by_map
                .get(&map_id)
                .map_or_else(|| [].iter(), |v| v.iter())
                .map(|o| o.id.join("."));

            trace!(
                "active trails ({}): {:?}",
                active_trails.len(),
                active_trails.take(MAX).collect::<Vec<_>>(),
            );
            trace!(
                "active trail points: {}",
                self.active_trails_of_current_map()
                    .map(|(_, trail)| trail.points.len())
                    .sum::<usize>(),
            )
        }
    }

    pub fn all_active_points_of_interest(
        &self,
    ) -> impl Iterator<Item = (&u32, &ActivePointOfInterest)> {
        self.active_points_of_interest_by_map
            .iter()
            .flat_map(|(map_id, trails)| {
                let only_map_id = [map_id].into_iter();

                only_map_id.zip(trails)
            })
    }

    pub fn active_points_of_interest_of_current_map(
        &self,
    ) -> impl Iterator<Item = (&u32, &ActivePointOfInterest)> {
        let only_current_map_id = [&self.current_map_id].into_iter().cycle();

        let points_of_interest = self
            .active_points_of_interest_by_map
            .get(&self.current_map_id)
            .map_or_else(|| [].iter(), |v| v.iter());

        only_current_map_id.zip(points_of_interest)
    }

    pub fn all_active_trails(&self) -> impl Iterator<Item = (&u32, &ActiveTrail)> {
        self.active_trails_by_map
            .iter()
            .flat_map(|(map_id, trails)| {
                let only_map_id = [map_id].into_iter();

                only_map_id.zip(trails)
            })
    }

    pub fn active_trails_of_current_map(&self) -> impl Iterator<Item = (&u32, &ActiveTrail)> {
        let only_current_map_id = [&self.current_map_id].into_iter().cycle();

        let trails = self
            .active_trails_by_map
            .get(&self.current_map_id)
            .map_or_else(|| [].iter(), |v| v.iter());

        only_current_map_id.zip(trails)
    }
}

#[derive(Debug)]
pub struct ActiveTrail<'a> {
    #[cfg(debug_assertions)]
    pub id: &'a Vec<String>,
    pub hash: u64,
    pub trail_width: TrailWidth,
    pub trail_color: TrailColor,
    pub points: &'a Vec<Point3>,
}

#[derive(Debug)]
pub struct ActivePointOfInterest<'a> {
    #[cfg(debug_assertions)]
    pub id: &'a Vec<String>,
    // TODO
    pub point: &'a Point3,
}
