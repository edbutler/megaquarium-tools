#lang errortrace racket

(provide lookup)

(require
  "data.rkt"
  "display.rkt"
  "localization.rkt"
  "serialize.rkt")

(define (do-lookup search-term)
  (define species (fuzzy-match-species search-term all-species))
  (cond
   [(empty? species)
    (printf "No species found matching '~a'.\n" search-term)]
   [else
    (for ([s (sort species string<? #:key (curry localize-species default-l10n))])
      (print-species default-l10n s))]))

(define (lookup program-name args)
  (command-line
    #:program program-name 
    #:argv args
    #:usage-help "Print information about animals whose names match <search-term>."
    #:args (search-term)
    (do-lookup search-term)))
