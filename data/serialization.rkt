; serializes to and from the tools yaml representation
#lang racket

(provide
  read-tank-yaml
  write-tank-yaml)

(require
  json
  yaml
  "game-data.rkt"
  "../lift.rkt"
  "../core.rkt")

(define (flip-pair x) (cons (cdr x) (car x)))

; -> (list of tank-spec?)
(define (read-tank-yaml port #:species species-list)
  (define data (read-yaml port))

  (define (make-spec pr)
    (define cnt (cdr pr))
    (define name (car pr))
    (define possible (fuzzy-match-species name species-list))
    (unless (= 1 (length possible)) (error (format "ambiguous/unkonwn species ~a" name)))
    (species-spec (first possible) cnt))

  (map
    (λ (pr)
      (tank-spec
        (car pr) ; name
        (map make-spec (hash->list (cdr pr)))))
    (hash->list (hash-ref data "tanks"))))

(define (write-tank-yaml aqrm out)
  (define (group->yaml animals)
    (cons (tweak-species-name (animal-species (first animals)))
          (length animals)))

  (define (exhibit->yaml exh)
    (define tnk (exhibit-tank exh))
    (define env (tank-environment tnk))
    (define animals (exhibit-animals exh))

    (define animal-counts (make-hash (map group->yaml (group-by animal-species animals))))

    (define tank-info
      (hasheq "type" (tweak-species-name (tank-kind-id (tank-type tnk)))
              "size" (tank-size tnk)))
              ;"temp" (symbol->string (environment-temperature env))
              ;"quality" (environment-quality env)
              ;"lighting" (tank-lighting tnk)))

    (cons (tank-name tnk)
          (hasheq "animals" animal-counts
                  "tank" tank-info)))

  (define (map-key key)
    (cond
     [(string=? key "tanks") "__tanks"]
     [else key]))

  (define (objective->yaml obj)
    (define c
      (match (objective-condition obj)
        [(species-condition id cnt)
         (hash "condition" "need-species"
               "id" (tweak-species-name id)
               "quantity" cnt)]
        [(restriction-condition r cnt)
         (hash "condition" "need-restriction" "restriction" (symbol->string r) "quantity" cnt)]
        [(type-condition t cnt)
         (hash "condition" "need-type" "type" (symbol->string t) "quantity" cnt)]
        [_ #f]))
    (hash "condition" c))

  (define (market->yaml mkt)
    (define (acquirable->yaml pr)
      (cons (tweak-species-name (car pr)) (cdr pr)))
    (hash
      "available" (map tweak-species-name (market-available mkt))
      "aquirable" (make-hash  (map acquirable->yaml (market-acquirable mkt)))
      "unlockable" (map tweak-species-name (market-unlockable mkt))))

  (write-yaml
    (hash
      "exhibits" (make-hash (map exhibit->yaml (aquarium-exhibits aqrm)))
      "market" (market->yaml (aquarium-market aqrm))
      "objectives" (map objective->yaml (aquarium-objectives aqrm)))
    out
    #:indent 2
    #:style 'block
    ; we want tanks to show up first
    #:sort-mapping-key (compose map-key car)
    #:sort-mapping string<?))
