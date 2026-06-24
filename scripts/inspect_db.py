#!/usr/bin/env python3
import os
import sqlite3

def main():
    # Resolve the default database path
    db_path = os.path.expanduser('~/.local/share/com.iptv.helper/iptv.db')
    
    if not os.path.exists(db_path):
        print(f"Error: Database file does not exist at: {db_path}")
        return

    print(f"==================================================")
    print(f" IPTV Helper DB Summary: {db_path}")
    print(f"==================================================")
    
    try:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        
        # Helper to query count
        def get_count(table_name):
            try:
                cursor.execute(f"SELECT COUNT(*) FROM {table_name}")
                return cursor.fetchone()[0]
            except sqlite3.OperationalError:
                return "Table not found"

        # 1. Profile Info
        print("\n--- Profiles ---")
        try:
            cursor.execute("SELECT id, name, server_url, username FROM profiles")
            profiles = cursor.fetchall()
            if not profiles:
                print("No profiles found.")
            for p in profiles:
                print(f"ID: {p[0]} | Name: {p[1]} | URL: {p[2]} | Username: {p[3]}")
        except sqlite3.OperationalError:
            print("Profiles table not found.")

        # 2. Sync Logs
        print("\n--- Sync Log Status ---")
        try:
            cursor.execute("SELECT data_type, fetched_at, item_count, last_error FROM sync_log")
            logs = cursor.fetchall()
            if not logs:
                print("No sync logs found.")
            for l in logs:
                err_str = f" | ERROR: {l[3]}" if l[3] else ""
                print(f"Type: {l[0]:<15} | Last Sync: {l[1]} | Count: {l[2]}{err_str}")
        except sqlite3.OperationalError:
            print("Sync log table not found.")

        # 3. Tables & Item Counts
        print("\n--- Database Row Counts ---")
        tables = [
            "live_categories",
            "live_streams",
            "vod_categories",
            "vod_streams",
            "series_categories",
            "series",
            "epg_entries"
        ]
        
        for table in tables:
            count = get_count(table)
            print(f"{table:<20} : {count}")
            
        conn.close()
    except Exception as e:
        print(f"An error occurred while inspecting the database: {e}")
        
    print(f"\n==================================================")

if __name__ == "__main__":
    main()
