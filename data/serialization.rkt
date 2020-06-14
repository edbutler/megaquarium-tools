; serializes to and from the tools yaml representation
#lang racket

(provide
  read-aquarium
  write-aquarium)

(require
  "game-data.rkt"
  "../lift.rkt"
  "../core.rkt")

(define (flip-pair x) (cons (cdr x) (car x)))

; -> (list of tank-spec?)
(define (read-aquarium port #:species species-list)
  (error "unimplemented"))

(define (write-aquarium aqrm out)
  (define (format-group animals)
    (cons (tweak-id/symbol (animal-species (first animals)))
          (length animals)))

  (define (format-tank tnk)
    `(tank
      #:type ,(tweak-id/symbol (tnktyp-id (tank-type tnk)))
      #:size ,(tank-size tnk)))

  (define (format-exhibit exh)
    (define tnk (exhibit-tank exh))
    (define env (tank-environment tnk))
    (define animals (exhibit-animals exh))

    (define animal-counts (map format-group (group-by animal-species animals)))

    `(exhibit
      #:name ,(tank-name tnk)
      #:tank ,(format-tank tnk)
      #:animals ,animal-counts))

  (define (objective->yaml obj)
    (define c
      (match (objective-condition obj)
        [(species-condition id cnt)
         (hash "condition" "need-species"
               "id" (tweak-id/string id)
               "quantity" cnt)]
        [(restriction-condition r cnt)
         (hash "condition" "need-restriction" "restriction" (symbol->string r) "quantity" cnt)]
        [(type-condition t cnt)
         (hash "condition" "need-type" "type" (symbol->string t) "quantity" cnt)]
        [_ #f]))
    (hash "condition" c))

  (define (market->yaml mkt)
    (define (acquirable->yaml pr)
      (cons (tweak-id/string (car pr)) (cdr pr)))
    (hash
      "available" (map tweak-id/string (market-available mkt))
      "aquirable" (make-hash  (map acquirable->yaml (market-acquirable mkt)))
      "unlockable" (map tweak-id/string (market-unlockable mkt))))

  (parameterize ([pretty-print-columns 40])
    (pretty-write
      `(aquarium #:exhibits ,(map format-exhibit (aquarium-exhibits aqrm)))
      )))

    ;(hash
    ;  "exhibits" (make-hash (map exhibit->yaml (aquarium-exhibits aqrm)))
    ;  "market" (market->yaml (aquarium-market aqrm))
    ;  "objectives" (map objective->yaml (aquarium-objectives aqrm)))
