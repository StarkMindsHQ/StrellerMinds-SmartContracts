# Learning Path Marketplace Contract

This contract enables instructors to list and sell learning paths, and students to discover and purchase them.

## Features

- **Listing**: Instructors can list learning paths with pricing and descriptions.
- **Purchase**: Students can purchase access to learning paths.
- **Rating**: Users can rate learning paths after purchase.
- **Discovery**: Query listed paths and user-owned paths.

## Functions

### `list_path(instructor, id, name, description, price)`
Lists a new learning path on the marketplace.

### `buy_path(buyer, id)`
Purchases access to a learning path.

### `rate_path(user, id, rating)`
Rates a purchased learning path (1-5 stars).

### `get_path(id)`
Retrieves details for a listed path.

### `get_user_paths(user)`
Retrieves all paths purchased by a user.
