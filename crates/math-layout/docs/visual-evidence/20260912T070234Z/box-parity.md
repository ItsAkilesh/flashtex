# Preview/PDF box parity report

Cases: 15; failed: 0 
Tolerance: 0.05 pt. Largest |Δ| over the corpus: PDF writer 0.00096 pt, CoreText placement 0.00000 pt.
Both sides consume the same runtime-v1 compile_result (text items per glyph with font-hints-v1, rule items per rules-v1). The PDF column is what crates/pdf wrote (`Td` origins, `re` rectangles, 3-decimal rounding); the CoreText column is what tools/coretext_boxes.swift drew (CTLineDraw per item at x_pt/baseline_y_pt, fill per rule). Faces are listed because the two sides substitute independently when Latin Modern is absent; glyph shapes are not compared here.
Run: `/private/tmp/claude-501/-Users-jay3332-Projects-flashtex/e30fd4a4-f46a-4c3f-a28c-cbb8617b4425/scratchpad/mv/run-20260912T070234Z`

### 01-stacked-fraction

- text items 5, rule items 2; max |Δ| vs compile_result: PDF writer 0.00049 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| a | 74.39103 | 75.43157 | 0.00043 | 0.00000 | LMRoman10-Italic | Times-Italic |
| b | 74.82874 | 84.26133 | 0.00033 | 0.00000 | LMRoman10-Italic | Times-Italic |
| c | 74.12104 | 96.84548 | 0.00048 | 0.00000 | LMRoman10-Italic | Times-Italic |
| = | 84.60089 | 88.64482 | 0.00018 | 0.00000 | LMRoman10-Regular | Times-Roman |
| 1 | 97.02638 | 88.64482 | 0.00038 | 0.00000 | LMRoman10-Regular | Times-Roman |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 74.39103 | 76.95067 | 4.49801 | 0.39848 | 0.00048 | 0.00000 |
| 73.19551 | 85.45678 | 6.88904 | 0.39848 | 0.00049 | 0.00000 |

### 02-sqrt-left-right

- text items 7, rule items 2; max |Δ| vs compile_result: PDF writer 0.00096 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| √ | 72.00000 | 77.78925 | 0.00025 | 0.00000 | LMRoman10-Regular | Times-Roman |
| x | 81.96267 | 86.94407 | 0.00033 | 0.00000 | LMRoman10-Italic | Times-Italic |
| + | 91.27142 | 86.94407 | 0.00042 | 0.00000 | LMRoman10-Regular | Times-Roman |
| ( | 103.03274 | 72.39848 | 0.00048 | 0.00000 | LMRoman10-Regular | Times-Roman |
| a | 111.56189 | 78.85632 | 0.00032 | 0.00000 | LMRoman10-Italic | Times-Italic |
| b | 112.14582 | 95.14474 | 0.00026 | 0.00000 | LMRoman10-Italic | Times-Italic |
| ) | 118.90235 | 72.39848 | 0.00048 | 0.00000 | LMRoman10-Regular | Times-Roman |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 81.96267 | 77.31106 | 6.65209 | 0.47819 | 0.00033 | 0.00000 |
| 111.56189 | 83.75604 | 6.14494 | 0.39848 | 0.00096 | 0.00000 |

### 03-sum-limits-scripts

- text items 8, rule items 0; max |Δ| vs compile_result: PDF writer 0.00048 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| n | 76.62615 | 76.42784 | 0.00016 | 0.00000 | LMRoman10-Italic | Times-Italic |
| ∑ | 72.00000 | 79.41662 | 0.00038 | 0.00000 | Symbol | Times-Roman |
| i | 72.34334 | 101.29006 | 0.00034 | 0.00000 | LMRoman10-Italic | Times-Italic |
| = | 75.22648 | 101.29006 | 0.00048 | 0.00000 | LMRoman10-Regular | Times-Roman |
| 1 | 81.81299 | 101.29006 | 0.00011 | 0.00000 | LMRoman10-Regular | Times-Roman |
| x | 88.38301 | 89.37932 | 0.00032 | 0.00000 | LMRoman10-Italic | Times-Italic |
| 2 | 95.03509 | 84.44313 | 0.00013 | 0.00000 | LMRoman10-Regular | Times-Roman |
| i | 95.03509 | 92.33483 | 0.00017 | 0.00000 | LMRoman10-Italic | Times-Italic |

### 04-tall-braces

- text items 16, rule items 5; max |Δ| vs compile_result: PDF writer 0.00076 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| { | 72.00000 | 72.00000 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 72.00000 | 80.96647 | 0.00047 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 72.00000 | 83.95529 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 72.00000 | 101.88822 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 72.00000 | 104.87705 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| a | 84.56297 | 77.43386 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| b | 84.94313 | 83.81791 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| c | 84.74616 | 91.52706 | 0.00016 | 0.00000 | LMRoman10-Italic | Times-Italic |
| d | 84.40139 | 99.85125 | 0.00039 | 0.00000 | LMRoman10-Italic | Times-Italic |
| e | 84.81964 | 105.42257 | 0.00043 | 0.00000 | LMRoman10-Italic | Times-Italic |
| f | 84.44223 | 111.80662 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| } | 92.30441 | 72.00000 | 0.00041 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 92.30441 | 80.96647 | 0.00047 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 92.30441 | 83.95529 | 0.00041 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 92.30441 | 101.88822 | 0.00041 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 92.30441 | 104.87705 | 0.00041 | 0.00000 | LMRoman10-Regular | Times-Roman |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 84.56297 | 78.50053 | 4.03417 | 0.39848 | 0.00048 | 0.00000 |
| 83.36745 | 84.21640 | 6.42520 | 0.39848 | 0.00060 | 0.00000 |
| 82.05121 | 92.72251 | 9.05769 | 0.39848 | 0.00049 | 0.00000 |
| 83.24672 | 101.37035 | 6.66667 | 0.39848 | 0.00065 | 0.00000 |
| 84.44223 | 106.48924 | 4.27564 | 0.39848 | 0.00076 | 0.00000 |

### 05-cube-root

- text items 4, rule items 2; max |Δ| vs compile_result: PDF writer 0.00052 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| 3 | 75.32083 | 83.40285 | 0.00042 | 0.00000 | LMRoman10-Regular | Times-Roman |
| √ | 72.33208 | 72.79697 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| a | 83.49025 | 79.82514 | 0.00025 | 0.00000 | LMRoman10-Italic | Times-Italic |
| b | 84.07418 | 96.11357 | 0.00043 | 0.00000 | LMRoman10-Italic | Times-Italic |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 82.29474 | 72.39848 | 8.53597 | 0.39848 | 0.00052 | 0.00000 |
| 83.49025 | 84.72487 | 6.14494 | 0.39848 | 0.00048 | 0.00000 |

### 06-lim-sin

- text items 11, rule items 1; max |Δ| vs compile_result: PDF writer 0.00072 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| l | 72.60557 | 88.02231 | 0.00043 | 0.00000 | LMRoman10-Regular | Times-Roman |
| i | 75.85723 | 88.02231 | 0.00031 | 0.00000 | LMRoman10-Regular | Times-Roman |
| m | 79.10889 | 88.02231 | 0.00031 | 0.00000 | LMRoman10-Regular | Times-Roman |
| x | 72.00000 | 94.81905 | 0.00011 | 0.00000 | LMRoman10-Italic | Times-Italic |
| → | 76.76690 | 94.81905 | 0.00011 | 0.00000 | LMRoman10-Regular | Times-Roman |
| 0 | 85.23526 | 94.81905 | 0.00026 | 0.00000 | LMRoman10-Regular | Times-Roman |
| s | 92.65746 | 79.93456 | 0.00046 | 0.00000 | LMRoman10-Regular | Times-Roman |
| i | 97.27482 | 79.93456 | 0.00044 | 0.00000 | LMRoman10-Regular | Times-Roman |
| n | 100.52648 | 79.93456 | 0.00048 | 0.00000 | LMRoman10-Regular | Times-Roman |
| x | 109.02230 | 79.93456 | 0.00044 | 0.00000 | LMRoman10-Italic | Times-Italic |
| x | 100.83988 | 96.22298 | 0.00017 | 0.00000 | LMRoman10-Italic | Times-Italic |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 92.65746 | 84.83428 | 23.01693 | 0.39848 | 0.00072 | 0.00000 |

### 07-tall-sqrt-braces

- text items 22, rule items 6; max |Δ| vs compile_result: PDF writer 0.00056 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| √ | 72.00000 | 72.79697 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| √ | 72.00000 | 78.37611 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| √ | 72.00000 | 84.35376 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| √ | 72.00000 | 90.33140 | 0.00040 | 0.00000 | LMRoman10-Regular | Times-Roman |
| √ | 72.00000 | 96.30905 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| √ | 72.00000 | 102.28669 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 82.51615 | 76.42920 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 82.51615 | 85.39567 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 82.51615 | 88.38449 | 0.00049 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 82.51615 | 106.31743 | 0.00043 | 0.00000 | LMRoman10-Regular | Times-Roman |
| { | 82.51615 | 109.30625 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| a | 95.07912 | 81.86306 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| b | 95.45928 | 88.24711 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| c | 95.26231 | 95.95626 | 0.00031 | 0.00000 | LMRoman10-Italic | Times-Italic |
| d | 94.91755 | 104.28046 | 0.00046 | 0.00000 | LMRoman10-Italic | Times-Italic |
| e | 95.33579 | 109.85177 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| f | 94.95839 | 116.23582 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| } | 102.82056 | 76.42920 | 0.00044 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 102.82056 | 85.39567 | 0.00044 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 102.82056 | 88.38449 | 0.00049 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 102.82056 | 106.31743 | 0.00044 | 0.00000 | LMRoman10-Regular | Times-Roman |
| } | 102.82056 | 109.30625 | 0.00044 | 0.00000 | LMRoman10-Regular | Times-Roman |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 82.51615 | 72.39848 | 29.16010 | 0.39848 | 0.00052 | 0.00000 |
| 95.07912 | 82.92974 | 4.03417 | 0.39848 | 0.00048 | 0.00000 |
| 93.88360 | 88.64560 | 6.42520 | 0.39848 | 0.00048 | 0.00000 |
| 92.56736 | 97.15172 | 9.05769 | 0.39848 | 0.00048 | 0.00000 |
| 93.76287 | 105.79955 | 6.66667 | 0.39848 | 0.00048 | 0.00000 |
| 94.95839 | 110.91844 | 4.27564 | 0.39848 | 0.00056 | 0.00000 |

### 08-nested-scripts

- text items 5, rule items 0; max |Δ| vs compile_result: PDF writer 0.00046 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| x | 72.00000 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Italic | Times-Italic |
| y | 78.65209 | 78.28754 | 0.00046 | 0.00000 | LMRoman10-Italic | Times-Italic |
| z | 83.12380 | 75.47475 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |
| i | 78.65209 | 86.70016 | 0.00016 | 0.00000 | LMRoman10-Italic | Times-Italic |
| j | 81.53523 | 87.91524 | 0.00042 | 0.00000 | LMRoman10-Italic | Times-Italic |

### 09-int-display

- text items 7, rule items 0; max |Δ| vs compile_result: PDF writer 0.00047 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| ∫ | 72.00000 | 73.98146 | 0.00046 | 0.00000 | Symbol | Times-Roman |
| 1 | 81.96266 | 77.13630 | 0.00034 | 0.00000 | LMRoman10-Regular | Times-Roman |
| 0 | 77.53482 | 96.61903 | 0.00018 | 0.00000 | LMRoman10-Regular | Times-Roman |
| f | 88.68747 | 88.03997 | 0.00047 | 0.00000 | LMRoman10-Italic | Times-Italic |
| ( | 95.73390 | 88.03997 | 0.00017 | 0.00000 | LMRoman10-Regular | Times-Roman |
| x | 100.28623 | 88.03997 | 0.00023 | 0.00000 | LMRoman10-Italic | Times-Italic |
| ) | 106.93832 | 88.03997 | 0.00032 | 0.00000 | LMRoman10-Regular | Times-Roman |

### 10-left-bracket-frac-squared

- text items 5, rule items 1; max |Δ| vs compile_result: PDF writer 0.00050 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| [ | 72.00000 | 74.37995 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| a | 78.45360 | 80.83778 | 0.00040 | 0.00000 | LMRoman10-Italic | Times-Italic |
| b | 79.03752 | 97.12620 | 0.00048 | 0.00000 | LMRoman10-Italic | Times-Italic |
| ] | 85.79405 | 74.37995 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| 2 | 91.05214 | 77.13630 | 0.00030 | 0.00000 | LMRoman10-Regular | Times-Roman |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 78.45360 | 85.73750 | 6.14494 | 0.39848 | 0.00050 | 0.00000 |

### 11-accents

- text items 5, rule items 0; max |Δ| vs compile_result: PDF writer 0.00039 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| ^ | 71.26664 | 83.95515 | 0.00036 | 0.00000 | LMRoman10-Regular | Times-Roman |
| ı | 72.00000 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Italic | Times-Italic |
| + | 78.39261 | 83.95517 | 0.00039 | 0.00000 | LMRoman10-Regular | Times-Roman |
| ⃗ | 89.95917 | 83.95515 | 0.00017 | 0.00000 | Times-Roman | Times-Italic |
| x | 90.15392 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Italic | Times-Italic |

### 12-sum-limits-inline

- text items 7, rule items 0; max |Δ| vs compile_result: PDF writer 0.00048 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| n | 76.28281 | 76.42784 | 0.00019 | 0.00000 | LMRoman10-Italic | Times-Italic |
| ∑ | 73.59384 | 78.42035 | 0.00036 | 0.00000 | Symbol | Times-Roman |
| i | 72.00000 | 95.31248 | 0.00048 | 0.00000 | LMRoman10-Italic | Times-Italic |
| = | 74.88314 | 95.31248 | 0.00048 | 0.00000 | LMRoman10-Regular | Times-Roman |
| 1 | 81.46965 | 95.31248 | 0.00048 | 0.00000 | LMRoman10-Regular | Times-Roman |
| x | 87.69633 | 86.39052 | 0.00048 | 0.00000 | LMRoman10-Italic | Times-Italic |
| i | 94.34841 | 88.18379 | 0.00041 | 0.00000 | LMRoman10-Italic | Times-Italic |

### 13-bigop-scripts-inline

- text items 7, rule items 0; max |Δ| vs compile_result: PDF writer 0.00043 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| ∏ | 72.00000 | 75.98500 | 0.00036 | 0.00000 | Symbol | Times-Roman |
| m | 81.40919 | 79.13984 | 0.00019 | 0.00000 | LMRoman10-Italic | Times-Italic |
| k | 81.40919 | 86.91068 | 0.00032 | 0.00000 | LMRoman10-Italic | Times-Italic |
| = | 86.03080 | 86.91068 | 0.00032 | 0.00000 | LMRoman10-Regular | Times-Roman |
| 1 | 92.61731 | 86.91068 | 0.00032 | 0.00000 | LMRoman10-Regular | Times-Roman |
| a | 99.34212 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Italic | Times-Italic |
| k | 105.48706 | 85.74843 | 0.00043 | 0.00000 | LMRoman10-Italic | Times-Italic |

### 14-mixed-text-math

- text items 16, rule items 0; max |Δ| vs compile_result: PDF writer 0.00047 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| L | 72.00000 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Regular | Times-Roman |
| e | 79.31417 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Regular | Times-Roman |
| t | 84.51683 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Regular | Times-Roman |
| x | 92.97115 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Italic | Times-Italic |
| 2 | 99.62323 | 79.61673 | 0.00027 | 0.00000 | LMRoman10-Regular | Times-Roman |
| + | 107.01221 | 83.95517 | 0.00021 | 0.00000 | LMRoman10-Regular | Times-Roman |
| y | 118.77353 | 83.95517 | 0.00047 | 0.00000 | LMRoman10-Italic | Times-Italic |
| 2 | 124.91017 | 79.61673 | 0.00027 | 0.00000 | LMRoman10-Regular | Times-Roman |
| = | 132.96331 | 83.95517 | 0.00031 | 0.00000 | LMRoman10-Regular | Times-Roman |
| z | 145.38879 | 83.95517 | 0.00021 | 0.00000 | LMRoman10-Italic | Times-Italic |
| 2 | 151.35940 | 79.61673 | 0.00040 | 0.00000 | LMRoman10-Regular | Times-Roman |
| h | 159.99371 | 83.95517 | 0.00029 | 0.00000 | LMRoman10-Regular | Times-Roman |
| o | 166.49703 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Regular | Times-Roman |
| l | 172.35002 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Regular | Times-Roman |
| d | 175.60168 | 83.95517 | 0.00032 | 0.00000 | LMRoman10-Regular | Times-Roman |
| . | 182.10500 | 83.95517 | 0.00017 | 0.00000 | LMRoman10-Regular | Times-Roman |

### 15-nested-fraction-sum

- text items 7, rule items 2; max |Δ| vs compile_result: PDF writer 0.00049 pt, CoreText 0.00000 pt
- verdict: PASS

| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |
|---|---|---|---|---|---|---|
| a | 73.72139 | 80.30218 | 0.00039 | 0.00000 | LMRoman10-Italic | Times-Italic |
| + | 82.52299 | 80.30218 | 0.00018 | 0.00000 | LMRoman10-Regular | Times-Roman |
| b | 94.28431 | 80.30218 | 0.00031 | 0.00000 | LMRoman10-Italic | Times-Italic |
| c | 74.73580 | 91.88348 | 0.00048 | 0.00000 | LMRoman10-Italic | Times-Italic |
| d | 74.39103 | 100.71323 | 0.00023 | 0.00000 | LMRoman10-Italic | Times-Italic |
| + | 82.60052 | 96.59060 | 0.00048 | 0.00000 | LMRoman10-Regular | Times-Roman |
| e | 94.36184 | 96.59060 | 0.00040 | 0.00000 | LMRoman10-Italic | Times-Italic |

| rule x | y | w | h | Δ pdf | Δ coretext |
|---|---|---|---|---|---|
| 73.19551 | 85.20190 | 26.59176 | 0.39848 | 0.00049 | 0.00000 |
| 74.39103 | 93.40257 | 4.35732 | 0.39848 | 0.00048 | 0.00000 |
