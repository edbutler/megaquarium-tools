#lang errortrace racket

(provide lookup)

(require
  "data.rkt"
  "display.rkt"
  "core.rkt")

(define (do-lookup search-term)
  (define data (read-game-data))
  (define species (fuzzy-match-species search-term (game-data-animals data)))
  (define l10n (game-data-localization data))
  (cond
   [(empty? species)
    (printf "No species found matching '~a'.\n" search-term)]
   [else
    (for ([s (sort species string<? #:key (curry localize-species l10n))])
      (print-species l10n s))]))

(define (lookup program-name args)
  (command-line
    #:program program-name 
    #:argv args
    #:usage-help "Print information about animals whose names match <search-term>."
    #:args (search-term)
    (do-lookup search-term)))
