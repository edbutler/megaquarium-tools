#lang racket

(provide info)

(require
  "display.rkt"
  "localization.rkt"
  "types.rkt"
  "data.rkt")

(define show-types? (make-parameter #f))
(define show-foods? (make-parameter #f))
(define localize? (make-parameter #f))

(define (localize-id id) (localize default-l10n id))

(define (print-ids header lst)
  ; use the same sort even if localizing so ids can be compared
  (set! lst (sort lst symbol<?))
  (define data
    (if (localize?)
      (map localize-id lst)
      (map symbol->string lst)))
  (pretty-display-data (hash header data)))

(define (show-types)
  (define all-types
    (remove-duplicates (map species-type all-species)))
  (print-ids "Types" all-types))

(define (show-foods)
  (define all-diets (map species-diet all-species))
  (define all-foods
    (remove-duplicates (filter-map (λ (d) (and (food? d) (food-type d))) all-diets)))
  (print-ids "Foods" all-foods))

(define (do-info)
  (define did-show-something #f)
  (when (show-types?)
    (set! did-show-something #t)
    (show-types))
  (when (show-foods?)
    (set! did-show-something #t)
    (show-foods))
  (unless did-show-something
    (printf "No flags set; set at least one flag to see output.\n")))

(define (info program-name args)
  (command-line
    #:program program-name 
    #:argv args
    #:once-each
    [("-l" "--localize") "Show localized strings instead of ids"
                         (localize? #t)]
    [("-t" "--types") "Show different animal types"
                      (show-types? #t)]
    [("-f" "--food") "Show different foods"
                      (show-foods? #t)]
    #:usage-help "Print information about various types of game objects."
    #:args ()
      (do-info)))