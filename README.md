[![Rust](https://github.com/betsegaw/just-lists/actions/workflows/rust.yml/badge.svg)](https://github.com/betsegaw/just-lists/actions/workflows/rust.yml)

# Just Lists - Simple Terminal Task Manager

**A powerful yet simple way to organize your tasks in the terminal**

Just Lists is a terminal-based task manager that helps you organize, manage, and track your tasks with an intuitive interface right in your command line.

![Animated walkethrough of Just-Lists features](https://github.com/Your_Repository_Name/docs/demo.gif)

## What You Can Do

- **Create and organize lists** - Build hierarchical task structures with parent and child items
- **Manage your tasks** - Mark items as complete, edit text, or delete when done
- **Navigate easily** - Use arrow keys to move between items and enter to expand/collapse
- **Work with nested lists** - Create complex task structures where items can have sub-items
- **Save your work** - All lists are automatically saved to JSON files

## Getting Started

### Installation

```bash
# Install via cargo (requires Rust)
cargo install just-lists

# Or download the binary from releases
```

### Running Just Lists

```bash
# Start with a sample list (no file specified)
jl

# Load from a specific file
jl your-tasks.json

# Create and save to a new file
jl my-new-list.json
```

**Important Note**: When starting with no file specified (`jl`), you'll see a sample list that is only displayed in the terminal and **not saved to any file**. Any changes made to this sample list will only persist during the current session and will be lost when you exit.

To save your lists, always provide a file path as an argument or use the "New" feature to create a new list file.

## Basic Usage

### Main Controls
- **↑↓** - Move up and down through items
- **Enter** - Expand/collapse items with children
- **Space** - Toggle item completion status
- **e** - Edit current item text
- **n** - Create a new top-level item
- **i** - Insert child item under the current item
- **d** - Delete the current item
- **j** - Focus on current item (shows only its children)
- **c** - Copy selected item
- **x** - Cut selected item  
- **v** - Paste item (adds as child of current item)
- **Esc** - Exit or return to main view

### Working with Nested Lists
1. Create parent items first
2. Use **Enter** to expand items and see their children
3. Use **i** to add child items under any item
4. Navigate between levels using arrow keys

### Breadcrumb Navigation Feature (Special)
The **`j` key** provides special **"focus view"** functionality:
- Press `j` to "focus" on the current item
- This shows only the children of that item in the main list view
- You can see the full path to your current location in the title bar
- This allows easy navigation between different branches of your task hierarchy

### Shared Item References (Special)
When you **copy** and then **paste** an item:
- The pasted item is actually the **same original item** - not a copy
- Changes to any instance are reflected everywhere the item appears
- This creates **shared references** to the same underlying data
- When you modify the pasted item, the original item also changes
- This allows for **data consistency** across all instances of an item

## Example Workflow

```bash
# Start with a fresh list
jl tasks.json

# Create your first task:
# Press 'n' to create new item, then type "Project Setup"

# Add subtasks:
# Press 'i' to insert child under "Project Setup"
# Type "Create project directory"

# Focus on a specific branch:
# Navigate to the "Project Setup" item
# Press 'j' to focus on just that branch
# Press 'j' again to return to full view

# Copy and paste an item:
# Select an item and press 'c' to copy
# Navigate to where you want to paste
# Press 'v' to paste (this creates a shared reference)

# Mark as complete:
# Press spacebar when cursor is on the item

# Delete tasks:
# Press 'd' to delete selected item
```

## Tips

- All lists are saved automatically to JSON files
- Start with a sample list or create your own file
- Use the path display at the top to see which list you're working with
- Items can be nested infinitely to create complex task hierarchies

## Getting Help

```bash
# View all available options
jl --help
```

Just Lists is designed to be simple and intuitive while providing powerful task management capabilities right in your terminal!
