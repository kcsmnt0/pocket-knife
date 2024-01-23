This project hopes to be a little image viewer tool for the Analogue Pocket. It doesn't work yet!

Project structure:

- `file-format`: library for the custom image archive file format
- `manager`: command-line tool for packing archives, depends on `file-format`
- `frontend`: Slint UI code, depends on `file-format`
- `backend-pocket`: main program code for Pocket, depends on `frontend`
- `backend-desktop`: main program code for desktop, depends on `frontend`

The desktop backend is meant purely for testing and debugging the UI. To compile the binary for the Pocket RISC-V core, run `make` in the `backend-pocket` folder.
