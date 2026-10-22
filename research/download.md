# Papers to download by hand

These 11 papers matter to the research, and no free copy turned up on 2026-10-22. Publisher sites, author homepages, university repositories, OpenAlex and the Wayback Machine were all searched. They need a library login.

Open each link in your normal browser and save the PDF in the folder of its heading, under the file name given. When some are in, ask Claude to convert them (`.md` with docling, `.txt` with pdftotext) and commit them.

ScienceDirect links (`sciencedirect.com/.../pdfft`) show the PDF only if the article is open access or your library subscribes. Older CAGD papers are often in Elsevier's free archive. The links go straight to the PDF; if one shows an article page instead, use its "View PDF" button.

## `research/papers/blends/`

| Paper | Link | Save as |
|---|---|---|
| Vida, Martin & Várady 1994, survey of blending methods | https://www.sciencedirect.com/science/article/pii/001044859490023X/pdfft | `Vida-Martin-Varady-1994-survey-blending-parametric-surfaces.pdf` |
| Várady & Rockwood 1997, setback vertex blending | https://www.sciencedirect.com/science/article/pii/S001044859600070X/pdfft | `Varady-Rockwood-1997-setback-vertex-blending.pdf` |
| Braid 1997, non-local blending of boundary models | https://www.sciencedirect.com/science/article/pii/S0010448596000383/pdfft | `Braid-1997-non-local-blending-boundary-models.pdf` |
| Klass & Kuhn 1992, fillet and surface intersections by rolling balls | https://www.sciencedirect.com/science/article/pii/016783969290016I/pdfft | `Klass-Kuhn-1992-fillet-surface-intersections-rolling-balls.pdf` |
| Farouki & Sverrisson 1996, approximating rolling-ball blends | https://www.sciencedirect.com/science/article/pii/0010448596000085/pdfft | `Farouki-Sverrisson-1996-approximation-rolling-ball-blends.pdf` |
| Lukács 1998, G1 variable-radius rolling-ball blends | https://www.sciencedirect.com/science/article/pii/S0167839698000065/pdfft | `Lukacs-1998-differential-geometry-G1-variable-radius-rolling-ball-blends.pdf` |
| Chuang, Lin & Hwang 1995, variable-radius blending (Springer) | https://link.springer.com/content/pdf/10.1007/BF02434038.pdf | `Chuang-Lin-Hwang-1995-variable-radius-blending-parametric-surfaces.pdf` |
| Pegna & Wolter 1992, curvature continuity of blends (ASME) | https://doi.org/10.1115/1.2916918 | `Pegna-Wolter-1992-curvature-continuity-blend-surfaces.pdf` |

## `research/papers/offsets_sweeps_lofts/`

| Paper | Link | Save as |
|---|---|---|
| Piegl & Tiller 2002, surface skinning revisited (Springer) | https://link.springer.com/content/pdf/10.1007/s003710100156.pdf | `Piegl-Tiller-2002-surface-skinning-revisited.pdf` |

## `research/papers/tessellation_mass_queries/`

| Paper | Link | Save as |
|---|---|---|
| Filip, Magedson & Markot 1986, surface algorithms using derivative bounds | https://www.sciencedirect.com/science/article/pii/0167839686900051/pdfft | `Filip-Magedson-Markot-1986-surface-algorithms-bounds-on-derivatives.pdf` |

## `research/papers/booleans/`

| Paper | Link | Save as |
|---|---|---|
| Urick et al. 2019, watertight Boolean operations | https://www.sciencedirect.com/science/article/pii/S0010448519302106/pdfft | `Urick-2019-watertight-boolean-operations.pdf` |

## Already fetched

33 papers from the original list were fetched on 2026-10-22, some as an earlier or preprint version:

- **intersection:** Krishnan & Manocha 1997, Manocha & Canny 1991, Sederberg & Zundel 1996, Dupont et al. 2008 I–III, Mourrain & Pavone 2009, Sherbrooke 1993 MIT thesis, Mukundan 2005 MIT thesis, Hu & Wallner 2005 (article-in-press copy, Tsinghua), Wang, Goldman & Tu 2003 (Wayback copy of the HKU file), Lin et al. 2014 (author's page, Zhejiang), Grandine & Klein 1997 (saved by hand)
- **blends:** Chuang & Hwang 1997 (NYCU repository); Dahl & Krasauskas 2012 as chapter 2 of Heidi Dahl's 2014 Oslo PhD thesis
- **offsets_sweeps_lofts:** Forsyth 1995
- **tessellation_mass_queries:** Gonzalez-Ochoa et al. 1998, Chen et al. 2006, Elber & Cohen 1990, Appel 1967, Johnson & Cohen 2005, Bénard, Hertzmann & Kass 2014, Hughes, Reali & Sangalli 2010 (as ICES Report 08-27)
- **booleans:** Attene 2020, EMBER 2022 (Wayback copy of the RWTH file)
- **exchange_healing_naming:** Bi et al. 2023, Marussig & Hughes 2018, Shen 2000 MIT thesis, Marcheix & Pierra 2002, Farjana & Han 2018 (images only, OCR'd), Capoyleas, Chen & Hoffmann 1996 (as Purdue tech report 94-011), Kripac 1997 (as its 1995 Solid Modeling version, same title), Benkő, Martin & Várady 2001 (accepted manuscript, Cardiff ORCA)

How they were found, for next time:
- **Chrome downloader:** `~/tools/playwright/fetch_pdf.sh` drives a visible Chrome window. It gets past the challenges ACM, BYU and Springer show to curl, but not a "verify you are human" checkbox such as ScienceDirect's.
- **MIT DSpace:** open the item page and click Download, because MIT rate-limits direct file requests.
- **Author copies:** search the exact title. Author homepages and university repositories often have a copy, and OpenAlex (`api.openalex.org/works/doi:<doi>`) lists repository copies.
- **Dead links:** the Wayback Machine (`web.archive.org/web/<year>id_/<url>`) often still has an author PDF whose link is now dead.

Alternatives for two papers still missing:
- **Urick et al. 2019:** Marussig et al. 2019, "Analysis-suitable CAD models based on watertight Boolean operations", is open access (https://onlinelibrary.wiley.com/doi/pdfdirect/10.1002/pamm.201900275).
- **Pegna & Wolter 1992:** the Filkins 1991 MIT thesis in `papers/blends/` covers its linkage-curve criterion.
