#!/usr/bin/env python3
"""
POP3 Client Example in Python

This script demonstrates how to use the Python poplib library to interact
with a POP3 server. It shows all major POP3 operations including:
- Connecting to a POP3 server
- Authentication
- Listing messages
- Retrieving messages
- Deleting messages
- Error handling

Usage:
    python3 pop3-client.py

Requirements:
    Python 3.6+
"""

import poplib
from email.parser import Parser
from email.policy import default
import sys


def example_basic_connection():
    """Example 1: Basic connection and authentication"""
    print("\n" + "="*50)
    print("EXAMPLE 1: Basic Connection and Authentication")
    print("="*50 + "\n")
    
    try:
        # Connect to POP3 server
        print("Connecting to localhost:110...")
        pop3_server = poplib.POP3("localhost", 110)
        
        # Read the server greeting
        greeting = pop3_server.getwelcome()
        print(f"Server greeting: {greeting.decode('utf-8')}")
        
        # Authenticate
        print("\nAuthenticating...")
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        print("Authentication successful!")
        
        # Close connection
        pop3_server.quit()
        print("Connection closed")
        
        return True
        
    except poplib.error_proto as e:
        print(f"POP3 protocol error: {e}")
        return False
    except Exception as e:
        print(f"Error: {e}")
        return False


def example_list_messages():
    """Example 2: List all messages in mailbox"""
    print("\n" + "="*50)
    print("EXAMPLE 2: List All Messages")
    print("="*50 + "\n")
    
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        
        # Get mailbox statistics
        message_count, mailbox_size = pop3_server.stat()
        print(f"Mailbox statistics:")
        print(f"  Messages: {message_count}")
        print(f"  Total size: {mailbox_size} bytes")
        
        # List all messages
        print(f"\nListing all messages:")
        response, message_list, octets = pop3_server.list()
        print(f"Response: {response.decode('utf-8')}")
        
        for msg in message_list:
            msg_info = msg.decode('utf-8')
            parts = msg_info.split()
            if len(parts) >= 2:
                msg_num, msg_size = parts[0], parts[1]
                print(f"  Message {msg_num}: {msg_size} bytes")
        
        pop3_server.quit()
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False


def example_retrieve_message():
    """Example 3: Retrieve and parse a message"""
    print("\n" + "="*50)
    print("EXAMPLE 3: Retrieve and Parse Message")
    print("="*50 + "\n")
    
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        
        message_count, _ = pop3_server.stat()
        
        if message_count > 0:
            print(f"Retrieving message 1...")
            
            # Retrieve message (returns response, lines, octets)
            response, lines, octets = pop3_server.retr(1)
            print(f"Response: {response.decode('utf-8')}")
            print(f"Size: {octets} bytes")
            
            # Parse the message
            msg_content = b'\n'.join(lines)
            msg = Parser(policy=default).parsestr(msg_content.decode('utf-8'))
            
            print(f"\n--- Message Details ---")
            print(f"From: {msg.get('From', 'N/A')}")
            print(f"To: {msg.get('To', 'N/A')}")
            print(f"Subject: {msg.get('Subject', 'N/A')}")
            print(f"Date: {msg.get('Date', 'N/A')}")
            
            # Get message body
            if msg.is_multipart():
                print("\n--- Message Body (multipart) ---")
                for part in msg.walk():
                    if part.get_content_type() == "text/plain":
                        body = part.get_payload(decode=True)
                        if body:
                            print(body.decode('utf-8', errors='ignore')[:500])
                            break
            else:
                print("\n--- Message Body ---")
                body = msg.get_payload(decode=True)
                if body:
                    print(body.decode('utf-8', errors='ignore')[:500])
        else:
            print("No messages in mailbox")
        
        pop3_server.quit()
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False


def example_retrieve_all_messages():
    """Example 4: Retrieve all messages"""
    print("\n" + "="*50)
    print("EXAMPLE 4: Retrieve All Messages")
    print("="*50 + "\n")
    
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        
        message_count, _ = pop3_server.stat()
        print(f"Retrieving {message_count} message(s)...\n")
        
        for i in range(1, message_count + 1):
            print(f"--- Message {i} ---")
            response, lines, octets = pop3_server.retr(i)
            
            msg_content = b'\n'.join(lines)
            msg = Parser(policy=default).parsestr(msg_content.decode('utf-8'))
            
            print(f"Subject: {msg.get('Subject', 'N/A')}")
            print(f"From: {msg.get('From', 'N/A')}")
            print(f"Size: {octets} bytes")
            print()
        
        pop3_server.quit()
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False


def example_delete_messages():
    """Example 5: Delete messages"""
    print("\n" + "="*50)
    print("EXAMPLE 5: Delete Messages")
    print("="*50 + "\n")
    
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        
        message_count, _ = pop3_server.stat()
        
        if message_count > 0:
            print(f"Marking message 1 for deletion...")
            response = pop3_server.dele(1)
            print(f"Response: {response.decode('utf-8')}")
            
            print("\nResetting (unmark deletion)...")
            response = pop3_server.rset()
            print(f"Response: {response.decode('utf-8')}")
            
            # Verify message still exists
            message_count_after_reset, _ = pop3_server.stat()
            print(f"Messages after reset: {message_count_after_reset}")
            
            # Mark for deletion again
            print("\nMarking message 1 for deletion again...")
            response = pop3_server.dele(1)
            print(f"Response: {response.decode('utf-8')}")
            
            print("\nNote: Deletion will be committed when we QUIT")
        else:
            print("No messages to delete")
        
        pop3_server.quit()
        print("Connection closed - deletion committed")
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False


def example_top_command():
    """Example 6: Use TOP command to get message headers"""
    print("\n" + "="*50)
    print("EXAMPLE 6: TOP Command (Headers Only)")
    print("="*50 + "\n")
    
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        
        message_count, _ = pop3_server.stat()
        
        if message_count > 0:
            # Get headers and first 0 lines of body
            print("Retrieving headers of message 1...")
            response, lines, octets = pop3_server.top(1, 0)
            print(f"Response: {response.decode('utf-8')}")
            
            print("\n--- Headers ---")
            for line in lines[:10]:  # Show first 10 lines
                print(line.decode('utf-8', errors='ignore'))
        else:
            print("No messages in mailbox")
        
        pop3_server.quit()
        return True
        
    except poplib.error_proto as e:
        print(f"TOP command not supported by server: {e}")
        return False
    except Exception as e:
        print(f"Error: {e}")
        return False


def example_uidl_command():
    """Example 7: Use UIDL command for unique message IDs"""
    print("\n" + "="*50)
    print("EXAMPLE 7: UIDL Command (Unique IDs)")
    print("="*50 + "\n")
    
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        
        message_count, _ = pop3_server.stat()
        
        if message_count > 0:
            print("Getting unique IDs for all messages...")
            response, uidl_list, octets = pop3_server.uidl()
            print(f"Response: {response.decode('utf-8')}")
            
            print("\n--- Unique IDs ---")
            for uidl in uidl_list:
                print(uidl.decode('utf-8'))
        else:
            print("No messages in mailbox")
        
        pop3_server.quit()
        return True
        
    except poplib.error_proto as e:
        print(f"UIDL command not supported by server: {e}")
        return False
    except Exception as e:
        print(f"Error: {e}")
        return False


def example_error_handling():
    """Example 8: Error handling scenarios"""
    print("\n" + "="*50)
    print("EXAMPLE 8: Error Handling")
    print("="*50 + "\n")
    
    # Test 1: Wrong credentials
    print("--- Test 1: Authentication with wrong credentials ---")
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("wronguser")
        pop3_server.pass_("wrongpass")
        pop3_server.quit()
        print("Unexpected: Authentication succeeded")
    except poplib.error_proto as e:
        print(f"Expected error: {e}")
    except Exception as e:
        print(f"Connection error (server not running?): {e}")
    
    # Test 2: Retrieve non-existent message
    print("\n--- Test 2: Retrieve non-existent message ---")
    try:
        pop3_server = poplib.POP3("localhost", 110)
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        
        # Try to retrieve message 99999 (should not exist)
        response, lines, octets = pop3_server.retr(99999)
        print("Unexpected: Retrieved non-existent message")
        pop3_server.quit()
    except poplib.error_proto as e:
        print(f"Expected error: {e}")
    except Exception as e:
        print(f"Connection error: {e}")
    
    # Test 3: Connection to wrong port
    print("\n--- Test 3: Connection to wrong port ---")
    try:
        pop3_server = poplib.POP3("localhost", 9999, timeout=2)
        print("Unexpected: Connection succeeded")
    except Exception as e:
        print(f"Expected error: {e}")


def example_secure_connection():
    """Example 9: Secure POP3 connection (POP3S)"""
    print("\n" + "="*50)
    print("EXAMPLE 9: Secure POP3 Connection (POP3S)")
    print("="*50 + "\n")
    
    try:
        # POP3S typically uses port 995
        print("Connecting to localhost:995 with SSL/TLS...")
        pop3_server = poplib.POP3_SSL("localhost", 995)
        
        greeting = pop3_server.getwelcome()
        print(f"Server greeting: {greeting.decode('utf-8')}")
        
        pop3_server.user("testuser")
        pop3_server.pass_("testpass")
        print("Secure authentication successful!")
        
        pop3_server.quit()
        return True
        
    except Exception as e:
        print(f"POP3S connection failed (expected if not configured): {e}")
        return False


def main():
    """Run all examples"""
    print("="*60)
    print("POP3 Client Examples in Python")
    print("="*60)
    print("\nThese examples demonstrate POP3 operations using Python's")
    print("poplib library.")
    print("\nNote: Examples assume a POP3 server is running on localhost:110")
    print("      with user 'testuser' and password 'testpass'.")
    print("\nIf the server is not running, you'll see connection errors.")
    
    # Run examples
    examples = [
        example_basic_connection,
        example_list_messages,
        example_retrieve_message,
        example_retrieve_all_messages,
        example_delete_messages,
        example_top_command,
        example_uidl_command,
        example_error_handling,
        example_secure_connection,
    ]
    
    for example_func in examples:
        try:
            example_func()
        except KeyboardInterrupt:
            print("\n\nInterrupted by user")
            sys.exit(0)
        except Exception as e:
            print(f"Unexpected error in {example_func.__name__}: {e}")
    
    print("\n" + "="*60)
    print("All examples completed!")
    print("="*60)


if __name__ == "__main__":
    main()
