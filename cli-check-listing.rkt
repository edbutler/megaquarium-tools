#lang racket

(provide check-listing)

(require
  "data.rkt"
  "core.rkt"
  "lift.rkt"
  "constraint.rkt")

(define (make-fishes specs #:id-suffix [id-suffix ""])
  (define (make sspec)
    (define spc (species-spec-species sspec))
    (define cnt (species-spec-count sspec))
    (define (name i) (format "~a-~a~a" (species-id spc) i id-suffix))
    (build-list cnt (λ (i) (animal (name i) spc))))
  (append-map make specs))

(define (minimum-viable-tank animals #:name [name #f] #:id [id 1])
  (make-tank
    id
    #:name (or name "tank")
    #:type (tank-kind #f #f #f (ormap (λ (a) (ormap rounded-tank? (animal-restrictions a))) animals))
    #:size (max (sum animal-final-size animals)
                (max-by (λ (a) (or (ormap (λ (r) (and (active-swimmer? r) (* (active-swimmer-multiplier r) (animal-final-size a))))
                                          (animal-restrictions a))
                                   0))
                        animals))
    #:environment (environment (environment-temperature (animal-environment (first animals)))
                               (apply max (map (compose1 environment-quality animal-environment) animals)))
    #:lighting (max-by (λ (a) (or (ormap (λ (r) (and (requires-light? r) (requires-light-amount r))) (animal-restrictions a)) 0)) animals)))

(define (format-violation data violtn)
  (define l10n (game-data-localization data))
  (match violtn
   [(cons (? animal? animl) restr)
    (format "\t~a has unmet requirement ~v\n" (localize-animal l10n animl) restr)]
   [(cons subj message)
    (format "\tTank has unmet requirement ~a\n" message)]))

(define (do-check-listing items)
  (unless (even? (length items)) (error "need even args"))

  (define data (read-game-data))

  (define species/counts
    (map
      (λ (i)
        (define search-str (list-ref items (* 2 i)))
        (define possible-species (fuzzy-match-species search-str (game-data-species data)))
        (when (empty? possible-species) (error (format "no matching species for '~a'" search-str)))
        (when (> (length possible-species) 1) (error (format "ambiguous match for '~a':\n\t~a" search-str (map species-id possible-species))))
        (define number (string->number (list-ref items (add1 (* 2 i)))))
        (species-spec (first possible-species) number))
      (range (/ (length items) 2))))

  (define animals (make-fishes species/counts))
  (when (empty? animals) (error "no animals given!"))
  (define tnk (minimum-viable-tank animals))
  (define dom (make-concrete-domain (list (cons tnk animals))))

  (printf "For contents:\n\t~a\n"
          (string-join (map (λ (s) (format "~a ~a"
                                           (species-spec-count s) (species-id (species-spec-species s))))
                            species/counts)
                       "\n\t"))
  (cond
   [(all-constraints-satisfied? dom)
    (printf "Required tank:\n\tsize: ~a\n\ttemp: ~a\n\tquality: ~a\n\tlight level: ~a\n\trounded? ~a\n"
            (tank-size tnk)
            (environment-temperature (tank-environment tnk))
            (environment-quality (tank-environment tnk))
            (tank-lighting tnk)
            (if (tank-kind-rounded? (tank-type tnk)) "yes" "no"))
    (define animals-grouped-by-food
      (group-by (compose1 food-type animal-diet)
                (filter (compose1 food? animal-diet) animals)))
    (printf "Required food:\n\t~a\n"
            (string-join
              (map (λ (lst) (format "~a ~a" (sum animal-required-food-amount lst) (food-type (animal-diet (first lst)))))
                   animals-grouped-by-food)
              "\n\t"))]
   [else
    (printf "Animals not compatible!\n")
    ; do a brain-dead de-duplication on the final printed strings
    (define strs
      (append
        (map (curry format-violation data) (find-violated-restrictions dom))
        (map (curry format-violation data) (find-violated-tank-constraints dom))))
    (for ([s (remove-duplicates strs)])
      (printf "~a" s))
    (printf "Minimum tank necessitated by other animals:\n\tsize: ~a\n\ttemp: ~a\n\tquality: ~a\n\tlight level: ~a\n\trounded? ~a\n"
            (tank-size tnk)
            (environment-temperature (tank-environment tnk))
            (environment-quality (tank-environment tnk))
            (tank-lighting tnk)
            (if (tank-kind-rounded? (tank-type tnk)) "yes" "no"))]))

(define (check-listing program-name args)
  (command-line
    #:program program-name 
    #:argv args
    #:usage-help
        "Finds the minimum viable tank that can hold a given a set of animals/counts."
        "If such a tank does not exist, prints the reasons for animal incompatibility."
        "Expects input as TODO"
    #:args (animal count . extra-animals/counts)
    (do-check-listing (cons animal (cons count extra-animals/counts)))))