import json
import os


def make_paths_absolute(data, base_path):
    if isinstance(data, dict):
        for key, value in data.items():
            if key == "root_module" or key == "path":
                data[key] = os.path.abspath(os.path.join(base_path, value))
            else:
                make_paths_absolute(value, base_path)
    elif isinstance(data, list):
        for item in data:
            make_paths_absolute(item, base_path)


def main():
    project_file = "rust-project.json"
    base_path = os.path.dirname(os.path.abspath(project_file))

    with open(project_file, "r") as file:
        data = json.load(file)

    make_paths_absolute(data, base_path)

    with open(project_file, "w") as file:
        json.dump(data, file, indent=4)


if __name__ == "__main__":
    main()
