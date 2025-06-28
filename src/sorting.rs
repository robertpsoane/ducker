use crate::docker::container::DockerContainer;
use crate::docker::image::DockerImage;
use crate::docker::volume::DockerVolume;
use crate::docker::network::DockerNetwork;

use std::cmp::Ordering;
use std::default::Default;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn toggle(self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerSortField {
    Name,
    Image,
    Status,
    Created,
    Ports,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSortField {
    Id,
    Name,
    Tag,
    Created,
    Size,
}

impl Default for ImageSortField {
    fn default() -> Self {
        ImageSortField::Name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeSortField {
    Name,
    Created,
    Driver,
    Mountpoint,
}

impl Default for VolumeSortField {
    fn default() -> Self {
        VolumeSortField::Name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSortField {
    Id,
    Name,
    Created,
    Scope,
    Driver,
}

impl Default for NetworkSortField {
    fn default() -> Self {
        NetworkSortField::Name
    }
}

#[derive(Debug, Clone)]
pub struct SortState<T> {
    pub field: T,
    pub order: SortOrder,
}

impl<T> SortState<T> {
    pub fn new(field: T) -> Self {
        Self {
            field,
            order: SortOrder::Ascending,
        }
    }

    pub fn toggle_or_set(&mut self, field: T)
    where
        T: PartialEq,
    {
        if self.field == field {
            self.order = self.order.toggle();
        } else {
            self.field = field;
            self.order = SortOrder::Ascending;
        }
    }

    pub fn is_field_sorted(&self, field: T) -> bool
    where
        T: PartialEq,
    {
        self.field == field
    }

    pub fn get_order_for_field(&self, field: T) -> Option<SortOrder>
    where
        T: PartialEq,
    {
        if self.field == field {
            Some(self.order)
        } else {
            None
        }
    }
}

impl<T> Default for SortState<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            field: T::default(),
            order: SortOrder::Ascending,
        }
    }
}

// Sorting functions for containers
pub fn sort_containers_by_name(a: &DockerContainer, b: &DockerContainer, order: SortOrder) -> Ordering {
    let cmp = a.names.cmp(&b.names);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_containers_by_image(a: &DockerContainer, b: &DockerContainer, order: SortOrder) -> Ordering {
    let cmp = a.image.cmp(&b.image);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_containers_by_status(a: &DockerContainer, b: &DockerContainer, order: SortOrder) -> Ordering {
    let cmp = a.status.cmp(&b.status);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_containers_by_created(a: &DockerContainer, b: &DockerContainer, order: SortOrder) -> Ordering {
    let cmp = a.created.cmp(&b.created);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_containers_by_ports(a: &DockerContainer, b: &DockerContainer, order: SortOrder) -> Ordering {
    let cmp = a.ports.cmp(&b.ports);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

// Sorting functions for images
pub fn sort_images_by_name(a: &DockerImage, b: &DockerImage, order: SortOrder) -> Ordering {
    let cmp = a.name.cmp(&b.name);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_images_by_tag(a: &DockerImage, b: &DockerImage, order: SortOrder) -> Ordering {
    let cmp = a.tag.cmp(&b.tag);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_images_by_created(a: &DockerImage, b: &DockerImage, order: SortOrder) -> Ordering {
    let cmp = a.created.cmp(&b.created);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_images_by_size(a: &DockerImage, b: &DockerImage, order: SortOrder) -> Ordering {
    let cmp = a.size.cmp(&b.size);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_images_by_id(a: &DockerImage, b: &DockerImage, order: SortOrder) -> Ordering {
    let cmp = a.id.cmp(&b.id);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

// Sorting functions for volumes
pub fn sort_volumes_by_name(a: &DockerVolume, b: &DockerVolume, order: SortOrder) -> Ordering {
    let cmp = a.name.cmp(&b.name);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_volumes_by_created(a: &DockerVolume, b: &DockerVolume, order: SortOrder) -> Ordering {
    let a_created = a.created_at.as_deref().unwrap_or("");
    let b_created = b.created_at.as_deref().unwrap_or("");
    let cmp = a_created.cmp(b_created);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_volumes_by_driver(a: &DockerVolume, b: &DockerVolume, order: SortOrder) -> Ordering {
    let cmp = a.driver.cmp(&b.driver);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_volumes_by_mountpoint(a: &DockerVolume, b: &DockerVolume, order: SortOrder) -> Ordering {
    let cmp = a.mountpoint.cmp(&b.mountpoint);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

// Sorting functions for networks
pub fn sort_networks_by_name(a: &DockerNetwork, b: &DockerNetwork, order: SortOrder) -> Ordering {
    let cmp = a.name.cmp(&b.name);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_networks_by_created(a: &DockerNetwork, b: &DockerNetwork, order: SortOrder) -> Ordering {
    let cmp = a.created_at.cmp(&b.created_at);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_networks_by_scope(a: &DockerNetwork, b: &DockerNetwork, order: SortOrder) -> Ordering {
    let cmp = a.scope.cmp(&b.scope);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_networks_by_driver(a: &DockerNetwork, b: &DockerNetwork, order: SortOrder) -> Ordering {
    let cmp = a.driver.cmp(&b.driver);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}

pub fn sort_networks_by_id(a: &DockerNetwork, b: &DockerNetwork, order: SortOrder) -> Ordering {
    let cmp = a.id.cmp(&b.id);
    match order {
        SortOrder::Ascending => cmp,
        SortOrder::Descending => cmp.reverse(),
    }
}
