import shutil
import os

# --- Configuration ---
SOURCE_DIR = 'R:/UB/templates/HW'

# --- Script Logic ---

def copy_directory_contents(source_dir=SOURCE_DIR, destination_dir=None):
    """
    Copies the contents (files and subdirectories) of a source directory 
    directly into the script's current working directory (CWD).
    """
    # 1. Get the current working directory (CWD)
    destination_dir = destination_dir or os.getcwd() 

    # Check if the source directory exists
    if not os.path.isdir(source_dir):
        print(f"❌ Error: Source directory not found at **{source_dir}**")
        return
    
    try:
        for item_name in os.listdir(source_dir):
            source_path = os.path.join(source_dir, item_name)
            destination_path = os.path.join(destination_dir, item_name)
            
            # Check if the item already exists in the destination
            if os.path.exists(destination_path):
                 print(f"⚠️ Warning: Skipping '{item_name}' as it already exists in the destination.")
                 continue

            if os.path.isdir(source_path):
                # If it's a directory, use copytree
                shutil.copytree(source_path, destination_path)
            else:
                # If it's a file, use copy2
                shutil.copy2(source_path, destination_path)

    except Exception as e:
        print(f"\n❌ An error occurred during copy: {e}")

if __name__ == "__main__":
    copy_directory_contents(SOURCE_DIR)