#lang racket

(provide (all-defined-out))

(require
  yaml
  "core.rkt")

(define (pretty-display-data data [out (current-output-port)])
  (write-yaml
    data
    out
    #:indent 2
    #:style 'block))

(define (format-temperature t)
  (cond
   [(equal? t warm-water) "warm"]
   [(equal? t cold-water) "cold"]
   [else "ERROR"]))

(define (format-size-stages stages)
  (define (format-stage s)
    (define sz (species-stage-size s))
    (match (species-stage-duration s)
     [#f (format "Size ~a" sz)]
     [x (format "Size ~a for ~a days" sz x)]))

  (string-join (map format-stage stages) " to "))

(define (format-property spc p)
  (match p
   [(bully) "Bully (no wimps)"]
   [_ "ERROR"]))

(define (format-restriction l10n spc r)
  (match r
   [(shoaler number) (format "Shoaler ~a" number)]
   [(predator type size)
    (if (and size (> size 0))
      (format "Preys on ~a of size ~a" (localize l10n type) size)
      (format "Preys on ~a" (localize l10n type)))]
   [(active-swimmer multiplier)
    (format "Active swimmer, needs size ~a"
            (* multiplier (species-final-size spc)))]
   [(dislikes-conspecifics) "Dislikes conspecifics (same species)"]
   [(dislikes-congeners) "Dislikes congeners (same type)"]
   [(only-congeners) "Congeners only (same type)"]
   [(rounded-tank) "Requires rounded tank"]
   [(dislikes-food-competitors) "Dislikes food competitors"]
   [(wimp) "Wimp (no bullies)"]
   [(dislikes-light) "Dislikes light"]
   [(requires-light amt) (format "Requires ~a light" amt)]
   [_ "ERROR"]))

(define (print-object l10n obj [out (current-output-port)])
  (define display-func
    (cond
      [(species? obj) display-species]
      [(tnktyp? obj) display-tank]
      [else (error "unknown object type")]))

  ; want a hash for nice printing, but want to keep them in order
  (define key-order '(id type size density water properties))

  (define yml (display-func l10n obj))

  (write-yaml
    (hash (localize l10n (game-object-template-id obj)) yml)
    out
    #:indent 2
    #:sort-mapping-key (λ (s) (index-of key-order (string->symbol (car s))))
    #:sort-mapping (λ (key1 key2) (and key1 key2 (< key1 key2)))
    #:style 'block))

; species? -> output-port? -> void?
(define (display-species l10n spc)
  (define yml (make-hash))
  (define (add key entry) (hash-set! yml (symbol->string key) entry))

  (add 'id (tweak-id/string (species-id spc)))
  (add 'type (localize l10n (species-type spc)))

  (let ([sze (species-size spc)])
    (when (> (species-final-size spc) 0)
      (add
        'size
        (format "~a~a"
          (format-size-stages (size-stages sze))
          (if (size-armored? sze) ", armored (size counts double for predators)" "")))))
  (let ([env (species-environment spc)])
    (add 'water (format "~a, ~a%"
                 (format-temperature (environment-temperature env))
                 (environment-quality env))))

  (add 'properties
       (append
         (for/list ([p (species-properties spc)])
           (format-property spc p))
         (for/list ([r (species-restrictions spc)])
           (format-restriction l10n spc r))))

  yml)

(define (display-tank l10n tnk)
  (define yml (make-hash))
  (define (add key entry) (hash-set! yml (symbol->string key) entry))

  (add 'id (tweak-id/string (tnktyp-id tnk)))
  (match-define (cons min-x min-y) (tnktyp-min-dimensions tnk))
  (match-define (cons max-x max-y) (tnktyp-max-dimensions tnk))
  (define density (tnktyp-volume-per-tile tnk))
  (define min-size (calculate-tank-size tnk min-x min-y))
  (define max-size (calculate-tank-size tnk max-x max-y))
  (add 'size
       (format "~ax~a to ~ax~a, ~a vol/tile (~a to ~a)"
               min-x min-y max-x max-y density min-size max-size))
  (when (tnktyp-rounded? tnk)
    (add 'properties (list "Rounded")))
  yml)

