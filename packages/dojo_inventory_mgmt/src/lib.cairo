pub mod store;

pub mod types {
    pub mod index;
}

pub mod events {
    pub mod index;
}

pub mod models {
    pub mod index;
    pub mod inventory;
}

pub mod components {
    pub mod inventory;
}

#[cfg(test)]
mod tests {
    pub mod setup;
    pub mod test_inventory;
}
