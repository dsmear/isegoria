# A2 probe, dtf-check (§13.18)

Audit: complete, 32 of 32 runs.
Precision check: 416 values, 0 failures.

## shared slopes

- C3 against C0, of 16 planned pairs: 16 comparable — greater 14, smaller 1, equal 1; not comparable 0 
- conditional on the 9 comparable pairs without a zero: greater 9, smaller 0, equal 0

- r0 (C3 generating DTF 0.5978044229590483): C0 DTF 0.13847275168454407 (counted classes), counted 2, pi [0.459, 0.541], gain 16.43, converged; C3 DTF 0.4120060809837205 (counted classes), counted 2, pi [0.329, 0.671], gain 17.5, converged — greater, gain minus C0's 1.07
- r1 (C3 generating DTF 0.6159756818719765): C0 DTF 0.8413682657231615 (counted classes), counted 2, pi [0.286, 0.714], gain 22.52, converged; C3 DTF 1.1139663846472792 (counted classes), counted 2, pi [0.294, 0.706], gain 45.72, converged — greater, gain minus C0's 23.2
- r2 (C3 generating DTF 0.594663255680098): C0 DTF 0.0 (one counted class), counted 1, pi [0.02, 0.98], gain 18.49, converged; C3 DTF 0.0 (one counted class), counted 1, pi [0.983, 0.017], gain 14.59, converged — equal, gain minus C0's -3.9
- r3 (C3 generating DTF 0.6007620610082341): C0 DTF 0.0 (one counted class), counted 1, pi [0.017, 0.983], gain 21.52, converged; C3 DTF 0.7933526513242677 (counted classes), counted 2, pi [0.061, 0.939], gain 16.35, converged — greater, gain minus C0's -5.16
- r4 (C3 generating DTF 0.6062788953261161): C0 DTF 0.49525030193586855 (counted classes), counted 2, pi [0.673, 0.327], gain 15.49, converged; C3 DTF 0.554671577521716 (counted classes), counted 2, pi [0.641, 0.359], gain 22.69, converged — greater, gain minus C0's 7.2
- r5 (C3 generating DTF 0.6153583869681802): C0 DTF 1.1273488879015165 (counted classes), counted 2, pi [0.053, 0.947], gain 16.28, converged; C3 DTF 1.323722644160443 (counted classes), counted 2, pi [0.061, 0.939], gain 20.99, converged — greater, gain minus C0's 4.71
- r6 (C3 generating DTF 0.6271791742868499): C0 DTF 0.6136521583816364 (counted classes), counted 2, pi [0.856, 0.144], gain 22.62, converged; C3 DTF 0.6306170467564611 (counted classes), counted 2, pi [0.217, 0.783], gain 25.67, converged — greater, gain minus C0's 3.06
- r7 (C3 generating DTF 0.6185353901494144): C0 DTF 0.10893824906557889 (counted classes), counted 2, pi [0.112, 0.888], gain 13.39, converged; C3 DTF 0.12891591691509705 (counted classes), counted 2, pi [0.834, 0.166], gain 14.27, converged — greater, gain minus C0's 0.88
- r8 (C3 generating DTF 0.568577519119808): C0 DTF 0.11709069698606736 (counted classes), counted 2, pi [0.586, 0.414], gain 26.77, converged; C3 DTF 0.5056169592982046 (counted classes), counted 2, pi [0.511, 0.489], gain 21.23, converged — greater, gain minus C0's -5.54
- r9 (C3 generating DTF 0.5829802980729105): C0 DTF 0.2293867547348413 (counted classes), counted 2, pi [0.099, 0.901], gain 15.27, converged; C3 DTF 0.0 (one counted class), counted 1, pi [0.041, 0.959], gain 20.92, converged — smaller, gain minus C0's 5.66
- r10 (C3 generating DTF 0.591432464039314): C0 DTF 0.35284969973083813 (counted classes), counted 2, pi [0.752, 0.248], gain 13.35, converged; C3 DTF 0.7450047054646092 (counted classes), counted 2, pi [0.06, 0.94], gain 14.03, converged — greater, gain minus C0's 0.68
- r11 (C3 generating DTF 0.6279230144514919): C0 DTF 0.0 (one counted class), counted 1, pi [0.048, 0.952], gain 29.62, converged; C3 DTF 0.4357840626901974 (counted classes), counted 2, pi [0.555, 0.445], gain 15.82, converged — greater, gain minus C0's -13.8
- r12 (C3 generating DTF 0.573807856388442): C0 DTF 0.0 (one counted class), counted 1, pi [0.023, 0.977], gain 11.38, converged; C3 DTF 0.6575475276909519 (counted classes), counted 2, pi [0.066, 0.934], gain 13.94, converged — greater, gain minus C0's 2.56
- r13 (C3 generating DTF 0.6144788582965366): C0 DTF 0.0 (one counted class), counted 1, pi [0.05, 0.95], gain 16.3, converged; C3 DTF 0.06636996659101639 (counted classes), counted 2, pi [0.193, 0.807], gain 18.4, converged — greater, gain minus C0's 2.11
- r14 (C3 generating DTF 0.5751256530320263): C0 DTF 0.5791076372029628 (counted classes), counted 2, pi [0.402, 0.598], gain 13.42, converged; C3 DTF 0.7404127497147845 (counted classes), counted 2, pi [0.403, 0.597], gain 19.14, converged — greater, gain minus C0's 5.72
- r15 (C3 generating DTF 0.5727862802256072): C0 DTF 0.0 (one counted class), counted 1, pi [0.96, 0.04], gain 16.93, converged; C3 DTF 0.7754510511552904 (counted classes), counted 2, pi [0.896, 0.104], gain 17.95, converged — greater, gain minus C0's 1.02

## per-class slopes

- C3 against C0, of 16 planned pairs: 15 comparable — greater 8, smaller 5, equal 2; not comparable 1 {'C0: candidate not converged': 1}
- conditional on the 9 comparable pairs without a zero: greater 6, smaller 3, equal 0

- r0 (C3 generating DTF 0.5978044229590483): C0 DTF 0.24842565628681826 (counted classes), counted 2, pi [0.089, 0.911], gain 51.53, converged; C3 DTF 0.22708608687158957 (counted classes), counted 2, pi [0.084, 0.916], gain 45.64, converged — smaller, gain minus C0's -5.88
- r1 (C3 generating DTF 0.6159756818719765): C0 DTF 0.7259172985412606 (counted classes), counted 2, pi [0.241, 0.759], gain 59.33, converged; C3 DTF 1.145116456122268 (counted classes), counted 2, pi [0.757, 0.243], gain 73.1, converged — greater, gain minus C0's 13.77
- r2 (C3 generating DTF 0.594663255680098): C0 DTF 0.2740907985365633 (counted classes), counted 2, pi [0.435, 0.565], gain 40.71, converged; C3 DTF 0.3122350252604715 (counted classes), counted 2, pi [0.112, 0.888], gain 28.24, converged — greater, gain minus C0's -12.47
- r3 (C3 generating DTF 0.6007620610082341): C0 DTF 0.6099327436258036 (counted classes), counted 2, pi [0.833, 0.167], gain 26.19, converged; C3 DTF 0.0 (one counted class), counted 1, pi [0.044, 0.956], gain 30.94, converged — smaller, gain minus C0's 4.75
- r4 (C3 generating DTF 0.6062788953261161): C0 DTF 0.39844152607657446 (counted classes), counted 2, pi [0.08, 0.92], gain 31.13, converged; C3 DTF 0.4168110107820853 (counted classes), counted 2, pi [0.555, 0.445], gain 28.06, converged — greater, gain minus C0's -3.06
- r5 (C3 generating DTF 0.6153583869681802): C0 DTF 0.0 (one counted class), counted 1, pi [0.971, 0.029], gain 36.94, converged; C3 DTF 0.0 (one counted class), counted 1, pi [0.983, 0.017], gain 38.0, converged — equal, gain minus C0's 1.06
- r6 (C3 generating DTF 0.6271791742868499): C0 DTF 0.0 (one counted class), counted 1, pi [0.043, 0.957], gain 42.13, converged; C3 DTF 1.2819111527073825 (counted classes), counted 2, pi [0.084, 0.916], gain 40.78, converged — greater, gain minus C0's -1.35
- r7 (C3 generating DTF 0.6185353901494144): C0 DTF 0.0 (one counted class), counted 1, pi [0.96, 0.04], gain 35.52, converged; C3 DTF 1.0351549449282853 (counted classes), counted 2, pi [0.065, 0.935], gain 38.49, converged — greater, gain minus C0's 2.97
- r8 (C3 generating DTF 0.568577519119808): C0 DTF 0.5229815914804455 (counted classes), counted 2, pi [0.93, 0.07], gain 43.68, converged; C3 DTF 0.6039945582721696 (counted classes), counted 2, pi [0.875, 0.125], gain 42.32, converged — greater, gain minus C0's -1.36
- r9 (C3 generating DTF 0.5829802980729105): C0 DTF 0.888274449280967 (counted classes), counted 2, pi [0.054, 0.946], gain 34.74, converged; C3 DTF 0.8457044606436642 (counted classes), counted 2, pi [0.057, 0.943], gain 29.19, converged — smaller, gain minus C0's -5.55
- r10 (C3 generating DTF 0.591432464039314): C0 DTF 0.4393184659196743 (counted classes), counted 2, pi [0.055, 0.945], gain 37.85, max_iters; C3 DTF 0.6406610545686097 (counted classes), counted 2, pi [0.877, 0.123], gain 28.95, converged — not comparable: {'C0': 'candidate not converged', 'C3': None}
- r11 (C3 generating DTF 0.6279230144514919): C0 DTF 0.2809424432615952 (counted classes), counted 2, pi [0.067, 0.933], gain 38.32, converged; C3 DTF 0.6477053574047257 (counted classes), counted 2, pi [0.124, 0.876], gain 39.01, converged — greater, gain minus C0's 0.69
- r12 (C3 generating DTF 0.573807856388442): C0 DTF 0.7117745886289517 (counted classes), counted 2, pi [0.938, 0.062], gain 32.13, converged; C3 DTF 0.0 (one counted class), counted 1, pi [0.964, 0.036], gain 35.28, converged — smaller, gain minus C0's 3.15
- r13 (C3 generating DTF 0.6144788582965366): C0 DTF 0.8073880382877549 (counted classes), counted 2, pi [0.913, 0.087], gain 35.93, converged; C3 DTF 0.7659250260800006 (counted classes), counted 2, pi [0.907, 0.093], gain 53.88, converged — smaller, gain minus C0's 17.95
- r14 (C3 generating DTF 0.5751256530320263): C0 DTF 0.6829044632473447 (counted classes), counted 2, pi [0.133, 0.867], gain 27.75, converged; C3 DTF 0.9382516330833659 (counted classes), counted 2, pi [0.12, 0.88], gain 40.59, converged — greater, gain minus C0's 12.84
- r15 (C3 generating DTF 0.5727862802256072): C0 DTF 0.0 (one counted class), counted 1, pi [0.043, 0.957], gain 24.41, converged; C3 DTF 0.0 (one counted class), counted 1, pi [0.048, 0.952], gain 33.43, converged — equal, gain minus C0's 9.01

The variants refit the same batches: they are not independent and are not summed.
