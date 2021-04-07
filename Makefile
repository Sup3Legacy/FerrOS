all:
	cargo run --release

doc:
	cargo doc --document-private-items --open

clean:
	cargo clean

sound:
	cargo run -- -soundhw pcspk -drive format=raw,file=disk.disk,index=2 -boot c

count:
	wc -l `find src -type f`

memory:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk.img,index=2 -boot c

memory2:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk.disk,index=2 -boot c

.PHONY: all clean count memory memory2

PANDOC = pandoc
PANDOC-TEMPLATE = report/template.latex
PANDOC-FLAGS = --template $(PANDOC-TEMPLATE) --listings --pdf-engine=xelatex -V geometry:a4paper -V geometry:margin=2cm
PANDOC-BEAMER-FLAGS = --template $(PANDOC-TEMPLATE) --listings --slide-level 2 --pdf-engine=xelatex

XELATEX = xelatex
BIBTEX = bibtex

PRESENTATION_SRC = $(shell find presentation -type f -name '*.md')
REPORT_SRC = $(shell find report -type f -name '*.md')
REPORT_PDF = report.pdf
MAIN_TEX = $(ARTIFACTS_DIR)/report.tex
REFERENCES_BIB = report/references.bib
PRESENTATION_PDF = presentation.pdf
ARTIFACTS_DIR = artifacts

all: report presentation
clean:
	rm -rf $(ARTIFACTS_DIR)

mk_artifacts_dir:
	mkdir -p $(ARTIFACTS_DIR)

report: mk_artifacts_dir $(ARTIFACTS_DIR)/$(REPORT_PDF)
presentation: mk_artifacts_dir $(ARTIFACTS_DIR)/$(PRESENTATION_PDF)

$(ARTIFACTS_DIR)/$(REPORT_PDF): $(REPORT_SRC)
	$(PANDOC) report/report.md $(PANDOC-FLAGS) -so $(MAIN_TEX)
	cp $(REFERENCES_BIB) $(ARTIFACTS_DIR)
	cd $(ARTIFACTS_DIR) && $(XELATEX) report.tex && $(BIBTEX) report && $(XELATEX) report.tex && mv main.pdf report.pdf

$(ARTIFACTS_DIR)/$(PRESENTATION_PDF): $(PRESENTATION_SRC)
	$(PANDOC) -t beamer presentation/slides.md $(PANDOC-BEAMER-FLAGS) -o $@
