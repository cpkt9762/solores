#!/usr/bin/env python3
"""
Script to add Anchor discriminator fields and update descriptions for IDL files.

This script processes Anchor IDL files that are missing discriminator fields,
calculating the correct 8-byte discriminator values for each instruction and
updating the description to indicate they were "Created with Anchor".
"""

import json
import hashlib
import shutil
import os
from typing import Dict, List, Any

def calculate_instruction_discriminator(name: str) -> List[int]:
    """
    Calculate Anchor instruction discriminator using the formula:
    sha256("global:{instruction_name}").slice(0, 8)
    
    Args:
        name: The instruction name
        
    Returns:
        List of 8 integers representing the discriminator bytes
    """
    hash_input = f"global:{name}"
    hash_bytes = hashlib.sha256(hash_input.encode()).digest()
    return list(hash_bytes[:8])

def calculate_account_discriminator(name: str) -> List[int]:
    """
    Calculate Anchor account discriminator using the formula:
    sha256("account:{account_name}").slice(0, 8)
    
    Args:
        name: The account name
        
    Returns:
        List of 8 integers representing the discriminator bytes
    """
    hash_input = f"account:{name}"
    hash_bytes = hashlib.sha256(hash_input.encode()).digest()
    return list(hash_bytes[:8])

def update_description(description: str) -> str:
    """
    Update description to include Anchor identifier if not already present.
    
    Args:
        description: Current description string
        
    Returns:
        Updated description with Anchor identifier
    """
    if "Created with Anchor" not in description:
        return f"{description} - Created with Anchor"
    return description

def process_idl_file(file_path: str) -> bool:
    """
    Process a single IDL file to add discriminators and update description.
    
    Args:
        file_path: Path to the IDL JSON file
        
    Returns:
        True if file was successfully processed, False otherwise
    """
    try:
        # Create backup
        backup_path = f"{file_path}.bak"
        shutil.copy2(file_path, backup_path)
        print(f"Created backup: {backup_path}")
        
        # Read the IDL file
        with open(file_path, 'r', encoding='utf-8') as f:
            idl_data = json.load(f)
        
        # Update description
        if 'metadata' in idl_data and 'description' in idl_data['metadata']:
            old_desc = idl_data['metadata']['description']
            new_desc = update_description(old_desc)
            idl_data['metadata']['description'] = new_desc
            print(f"Updated description: {old_desc} -> {new_desc}")
        
        # Process instructions
        if 'instructions' in idl_data:
            for instruction in idl_data['instructions']:
                if 'name' in instruction:
                    # Only add discriminator if it doesn't exist
                    if 'discriminator' not in instruction:
                        discriminator = calculate_instruction_discriminator(instruction['name'])
                        instruction['discriminator'] = discriminator
                        print(f"Added discriminator for instruction '{instruction['name']}': {discriminator}")
        
        # Process accounts if they exist at top level
        if 'accounts' in idl_data:
            for account in idl_data['accounts']:
                if 'name' in account:
                    # Only add discriminator if it doesn't exist
                    if 'discriminator' not in account:
                        discriminator = calculate_account_discriminator(account['name'])
                        account['discriminator'] = discriminator
                        print(f"Added discriminator for account '{account['name']}': {discriminator}")
        
        # Write back to file with proper formatting
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(idl_data, f, indent=2, ensure_ascii=False)
        
        print(f"Successfully processed: {file_path}")
        return True
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        # Restore backup if it exists
        backup_path = f"{file_path}.bak"
        if os.path.exists(backup_path):
            shutil.copy2(backup_path, file_path)
            print(f"Restored from backup due to error")
        return False

def main():
    """Main function to process all target IDL files."""
    
    # Files that need discriminator fields added
    target_files = [
        "idls/pump-fun-idl.json",
        "idls/lifinity.json", 
        "idls/moonshot.json",
        "idls/squads_multisig_program.json",
        "idls/whirlpool.json"
    ]
    
    print("Starting Anchor discriminator and description update process...")
    
    success_count = 0
    total_count = len(target_files)
    
    for file_path in target_files:
        if os.path.exists(file_path):
            print(f"\n--- Processing {file_path} ---")
            if process_idl_file(file_path):
                success_count += 1
            else:
                print(f"Failed to process {file_path}")
        else:
            print(f"File not found: {file_path}")
    
    print(f"\n=== Summary ===")
    print(f"Successfully processed: {success_count}/{total_count} files")
    
    if success_count == total_count:
        print("All files processed successfully!")
    else:
        print(f"Some files failed to process. Check error messages above.")

if __name__ == "__main__":
    main()