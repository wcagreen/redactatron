# Redactatron 9000 - UI User Guide

Welcome to Redactatron 9000! This guide will help you navigate the application and effectively redact sensitive information from your documents.

## Table of Contents

- [Starting the Application](#starting-the-application)
- [Home Screen](#home-screen)
- [Editor Interface](#editor-interface)
  - [Left Sidebar](#left-sidebar)
  - [Main Viewer](#main-viewer)
  - [Search Results Panel](#search-results-panel)
- [How to Redact Documents](#how-to-redact-documents)
- [Search and Navigation](#search-and-navigation)
- [Exporting Redacted Files](#exporting-redacted-files)
- [Tips and Tricks](#tips-and-tricks)
- [Troubleshooting](#troubleshooting)

## Starting the Application

1. Launch **Redactatron** by running the executable
2. The application window will open with the Home screen
3. Check the application logs if you encounter any issues (see [Application Logs](#troubleshooting) section)

## Home Screen

The Home screen is where you begin by loading documents for redaction.

### Loading Documents

There are two ways to load files:

#### Method 1: Drag and Drop
- Drag files directly from your file explorer onto the **"📁 Drag and Drop Files Here"** area
- You can drag multiple files at once
- The application will display all dragged files in your redaction session

#### Method 2: Browse Button
- Click the **"📂 Click to Browse"** button
- Select one or more files from the file browser dialog
- Supported formats:
  - **Documents**: .pdf, .docx (Word documents)
  - **Images**: .jpg, .jpeg, .png, .gif, .webp

### Starting Redaction

Once you've loaded your files:
1. Click the **"📋 Submit Files"** button
2. The first document will open in the Editor view
3. You can now begin marking areas for redaction

## Editor Interface

The editor interface is divided into three main sections:

### Left Sidebar

The left sidebar provides file management, search functionality, and redaction controls.

#### Files Section
- **File List**: Shows all loaded documents
- Click on any file to switch to it
- The current file is highlighted
- Navigate between multiple documents in one session

#### Search & Redact Section

**Search Feature**:
- Type keywords or phrases in the search box
- Click **"🔍 Search"** or press Enter
- The application will find all instances across the document
- Results appear in the right panel (see [Search Results Panel](#search-results-panel))
- Click **"Clear Search"** to reset the search

**Search Tips**:
- Search is case-insensitive
- Use partial terms to find variations (e.g., "email" finds "Email", "EMAIL", etc.)
- Results highlight specific text locations in the document

#### Redaction Section

**Areas Marked**: Displays how many redaction areas you've marked on the current page/file

**Redaction Mode Toggle** ✏️:
- **OFF** (Gray): Left-click drag to pan/navigate the document
- **ON** (Red): Left-click drag to create redaction boxes
- Click the button to toggle between modes

**Instructions**:
- When Redaction Mode is **ON**: "Left-click drag to create redaction"
- When Redaction Mode is **OFF**: "Left-click drag to pan"

**Clear Current Page**:
- Removes all redaction marks from the current page (PDFs) or entire file (images)
- Cannot be undone, so verify before clearing

**Export Redacted Document** 💾:
- Saves your redacted file (see [Exporting Redacted Files](#exporting-redacted-files))

### Main Viewer

The central area displays your document for viewing and marking redactions.

#### Navigation Controls

**For PDFs**:
- **Page Navigation**: Use arrow buttons or page number input at the bottom to navigate between pages
- **Zoom**: Scroll with mouse wheel to zoom in/out
- **Pan**: Drag with left mouse button (when Redaction Mode is OFF) to move around the document
- **Current Page Display**: Shows "Page X of Y"

**For Images**:
- **Zoom**: Scroll with mouse wheel to zoom in/out
- **Pan**: Drag with left mouse button to navigate
- Single image display

#### Creating Redactions

1. **Enable Redaction Mode**: Toggle the "✏️ Redaction Mode" button in the sidebar (turns red when ON)
2. **Click and Drag**: Left-click and drag on the document to create a black redaction box
3. **Multiple Redactions**: Create as many redaction boxes as needed
4. **Redaction Appears as**: Solid black rectangle covering the sensitive content

#### Managing Redactions

**Hover and Delete**:
- Hover your mouse over a redaction box
- Right-click on the hovered redaction
- Select the delete option to remove that specific redaction

**Viewing Redactions**:
- Redacted areas appear as solid black boxes
- They are permanent once exported (using rasterization)

### Search Results Panel

Appears on the right side when search results are found.

**Results Display**:
- Shows total number of matches found
- Results grouped by page number
- Each result shows a preview of the surrounding text (truncated at 40 characters)

**Navigation**:
- **◀ Previous Button**: Jump to the previous search result
- **▶ Next Button**: Jump to the next search result
- **Click on Result**: Select and navigate to that specific result

**Active Result Highlighting**:
- The currently active search result is highlighted in blue
- The document automatically scrolls to show the active result
- Useful for quickly locating and redacting multiple instances

## How to Redact Documents

### Step-by-Step Redaction Process

1. **Load Your Document**: Use the Home screen to load files (see [Home Screen](#home-screen))
2. **Navigate to Content**: 
   - For PDFs: Use page navigation to find the sensitive content
   - Zoom and pan as needed to get a clear view
3. **Enable Redaction Mode**: Click "✏️ Redaction Mode: OFF" to turn it ON (button turns red)
4. **Mark Areas**:
   - Click and drag to create black boxes over sensitive information
   - Mark all areas that need redaction
5. **Review Your Work**: 
   - Scroll through the document to verify all sensitive areas are marked
   - Use page navigation to check all pages
6. **Delete Mistakes** (if needed):
   - Right-click on incorrect redactions
   - Select delete to remove them
   - Add new redactions if needed
7. **Move to Next File**: Click on the next file in the left sidebar
8. **Export When Done**: See [Exporting Redacted Files](#exporting-redacted-files)

### Quick Tips for Effective Redaction

- **Be Thorough**: Mark the entire text area, not just individual letters
- **Use Search Feature**: Use "🔍 Search" to find all instances automatically
- **Review Pages**: Scroll through entire documents to catch all sensitive data
- **Zoom for Precision**: Zoom in when marking small text areas
- **For Large Areas**: Make multiple redactions for complex layouts

## Search and Navigation

### Using the Search Feature

1. **Type Your Search Term**:
   - In the left sidebar, enter a keyword (e.g., "confidential", "email@domain.com", "SSN")
   - The search is case-insensitive

2. **Execute Search**:
   - Click **"🔍 Search"** button or press Enter
   - Wait for results to appear in the right panel

3. **Navigate Results**:
   - Results are grouped by page
   - Click any result to jump to that location
   - Use **◀** and **▶** buttons to navigate between results

4. **Quick Redaction**:
   - After finding a search result, enable Redaction Mode
   - Mark the text area for redaction
   - Use search navigation to find the next instance

### Document Navigation

**For PDF Documents**:
- **Page Input**: Enter a page number and press Enter to jump directly to that page
- **Arrow Buttons**: Navigate one page at a time
- **Page Counter**: Shows current page and total pages (e.g., "Page 5 of 25")

**For Image Files**:
- Single image display (no page navigation)
- Use zoom and pan to navigate within the image

## Exporting Redacted Files

### Initiating Export

1. **Mark All Redactions**: Ensure all sensitive content is marked (see [How to Redact Documents](#how-to-redact-documents))
2. **Click "💾 Export Redacted Document"** in the left sidebar
3. The export process will begin

### Export Dialogs

#### No Redactions Warning
If you click export without any redactions:
- A dialog appears: "⚠️ No redactions detected for this file"
- Message states: "Please mark areas to redact before exporting"
- Click **"OK"** to dismiss and return to redaction mode

#### DPI Selection (PDF Export)
When exporting PDFs, you'll see a dialog:

**DPI Level Options**:
- The default is **150 DPI** (recommended for most use cases)
- **Higher DPI** (e.g., 600):
  - Pros: Better quality, higher resolution
  - Cons: Larger file size
- **Lower DPI** (e.g., 100):
  - Pros: Smaller file size
  - Cons: Reduced quality

**How to Select**:
1. Review the DPI recommendations in the dialog
2. Enter your preferred DPI value
3. Click **"Export"** to proceed

#### Save Location
- Choose where to save your redacted document
- The application will create a new file with your redactions applied
- Original file remains unchanged

#### Export Success
- A confirmation dialog appears when export completes
- Shows the filename of the exported document
- Click **"OK"** to dismiss

### Export Behavior

- **PDFs**: Converted to rasterized format (images embedded in PDF) - permanently removes redacted content
- **Documents**: Converted to PDF before rasterization
- **Images**: Embedded as rasterized image in PNG output

## Tips and Tricks

### Efficiency Tips

1. **Batch Process Multiple Files**: Load all documents at once and process them sequentially
2. **Use Search for Patterns**: Search for common sensitive info (emails, phone patterns, etc.)
3. **Zoom for Accuracy**: Zoom to 100-150% when marking small or precise areas
4. **Check All Pages**: Always scroll through the entire document before exporting

### Best Practices

1. **Review Before Export**: Double-check all redacted areas look correct
2. **Test with Less Important Files First**: Practice on non-critical documents
3. **Keep Original Files**: The app creates new files, so originals are always preserved
4. **Use High DPI for Important Documents**: Use 300/600 DPI for documents that require high quality
5. **Mark Complete Sections**: Rather than individual words, mark entire paragraphs when appropriate

### Keyboard Shortcuts

- **Enter**: Execute search (when search field is focused)
- **Mouse Wheel**: Zoom in/out on documents
- **Left-click Drag**: Create redactions (when mode is ON) or pan (when mode is OFF)
- **Right-click**: Delete redaction boxes

## Troubleshooting

### Common Issues and Solutions

#### Document Fails to Load
**Problem**: "Failed to load PDF file" or document doesn't appear

**Solutions**:
1. Verify the file is not corrupted by opening it in another application
2. Check that pdfium is installed and in your system PATH
3. Try with a different file to isolate the issue
4. Check the application logs for detailed error information

#### Word Document Won't Convert
**Problem**: .docx file fails to convert

**Solutions**:
1. Verify **Pandoc** is installed: [Download Pandoc](https://pandoc.org/installing.html)
2. Verify **MiKTeX** is installed (provides pdflatex): [Download MiKTeX](https://miktex.org/download)
3. Try converting the document manually with Pandoc:
   ```bash
   pandoc input.docx -o output.pdf --pdf-engine=pdflatex
   ```
4. Check if the document has unusual formatting or corruption
5. Ensure both Pandoc and MiKTeX are in your system PATH

#### Search Returns No Results
**Problem**: Search finds nothing when content exists

**Solutions**:
1. Try searching for partial terms.
2. Check that you typed the search term correctly
3. Verify the PDF is selectable text (some scanned PDFs may not support searching)
4. Try searching for simpler terms to test functionality

#### Redactions Look Incorrect
**Problem**: Redaction boxes don't cover the right area or appear misaligned

**Solutions**:
1. Right-click and delete the incorrect redaction
2. Adjust zoom level to 100% for precise marking
3. Create a new redaction that properly covers the sensitive content
4. Ensure you marked the entire sensitive area

#### Export Fails or Hangs
**Problem**: Export button doesn't respond or process hangs

**Solutions**:
1. Verify you have redactions marked (see [No Redactions Warning](#no-redactions-warning))
2. Try exporting a smaller section first
3. Close and restart the application
4. Check system disk space
5. Review logs for detailed error messages

### Checking Application Logs

Logs contain detailed information about all operations and errors:

**Log File Location**:
- **Windows**: `C:\Users\{YourUsername}\AppData\Local\redactatron\data\logs\redactor.log`
- **macOS**: `~/Library/Application Support/redactatron/logs/redactor.log`
- **Linux**: `~/.local/share/redactatron/logs/redactor.log`
- **Fallback**: `redactor.log` in your current working directory

**How to Review Logs**:
1. Navigate to the log file location above
2. Open `redactor.log` with any text editor
3. Look for entries with timestamps matching when the issue occurred
4. Search for "ERROR" or "error" to find problems
5. Review the context around error messages for details

**What Logs Contain**:
- File loading operations
- Conversion processes
- Redaction actions
- Export operations
- Error messages with details
- Performance information

## Getting Help

If you encounter issues not covered in this guide:

1. **Check the Logs**: Review application logs (see [Checking Application Logs](#checking-application-logs))
2. **Verify Prerequisites**: Ensure Pandoc, MiKTeX, and pdfium are installed
3. **Try Again**: Close and restart the application
4. **Report Issues**: Visit the [GitHub Repository](https://github.com/wcagreen/redactatron) to report bugs
5. **Check Documentation**: Review the main [README.md](../README.md) for additional information

