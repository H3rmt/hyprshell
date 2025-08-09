.PHONY: all clean combine prepare copy-defs build

SRCS = key-press.cpp layer-change.cpp init.cpp exit.cpp main.cpp
OUTDIR = out
OUTNAME = hyprshell.so
TARGET = $(OUTDIR)/$(OUTNAME)
HEADERS = globals.hpp handlers.hpp defs.hpp defs-test.hpp

CXXFLAGS = -shared -fPIC --no-gnu-unique -std=c++2b -O2 -I/usr/include/pixman-1

all: prepare build

prepare: combine copy-defs

clean:
	rm -rf $(OUTDIR)

combine:
# Combine all source files into one for easier compilation
	mkdir -p $(OUTDIR)
	rm -f $(OUTDIR)/all.cpp
	for src in $(SRCS); do \
		printf "\n\n// $$src \n" >> $(OUTDIR)/all.cpp; \
		cat $$src >> $(OUTDIR)/all.cpp; \
	done

copy-defs:
# Copy header files to the out directory
	cp $(HEADERS) $(OUTDIR)/

build:
# Compile the combined source file into a shared library
	g++ $(CXXFLAGS) -o $(TARGET) $(OUTDIR)/all.cpp