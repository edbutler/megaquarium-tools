#lang errortrace racket

(provide lookup)

(define search-class (make-parameter 'animal))

(require
  "data.rkt"
  "display.rkt"
  "core.rkt")

(define (do-lookup class search-term)
  (define data (read-game-data))
  (define objects
    (match class
      ['animal (game-data-species data)]
      ['animal (game-data-tanks data)]))
  (define lst (fuzzy-match-species search-term objects))
  (define l10n (game-data-localization data))
  (cond
   [(empty? lst)
    (printf "No objects found matching '~a'.\n" search-term)]
   [else
    (for ([s (sort lst string<? #:key (λ (obj) (localize l10n (game-object-template-id obj))))])
      (print-species l10n s))]))

(define (lookup program-name args)
  (command-line
    #:program program-name 
    #:argv args
    #:once-any
    [("-a" "--animal") "Search animals (default)" (search-class 'animal)]
    [("-t" "--tank") "Search tanks" (search-class 'tank)]
    #:usage-help "Print information about game objects whose names match <search-term>."
                 "Using an empty search term shows all objects."
    #:args (search-term)
    (do-lookup (search-class) search-term)))
